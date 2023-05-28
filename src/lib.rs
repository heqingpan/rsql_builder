

use serde_json::{json, Value};

pub trait IBuilder {
    fn build(&mut self)  -> (String,Vec<Value>);
}

#[derive(Debug,Clone)]
enum InnerSql<'a>{
    Value(String),
    Ref(&'a str)
}

fn pre_trim(prefix:&str,value:&str) -> usize{
    if let Some(i)=value.find(|c| !char::is_whitespace(c)){
        let len = value.len();
        let fix_len = prefix.len();
        if fix_len==0 || len - i < fix_len {
            return i;
        }
        if prefix.to_ascii_lowercase()==value[i..(i+fix_len)].to_ascii_lowercase() {
            return i+fix_len;
        }
    }
    0
}

fn suf_trim(suffix:&str,value:&str) -> usize{
    let len = value.len();
    if let Some(i)=value.rfind(|c| !char::is_whitespace(c)){
        let fix_len = suffix.len();
        if fix_len==0 || i < fix_len {
            return i+1;
        }
        if suffix.to_ascii_lowercase()==value[(i-fix_len+1)..(i+1)].to_ascii_lowercase() {
            return i-fix_len;
        }
    }
    len 
}

fn sql_trim(prefix:&str,suffix:&str,value:&str) -> (usize,usize){
    (pre_trim(prefix, value),suf_trim(suffix, value))
}
fn sql_trim_string(prefix:&str,suffix:&str,value:&str) ->String {
    let (s,e) = sql_trim(prefix, suffix, value);
    value[s..e].to_owned()
}


#[derive(Debug,Clone)]
pub enum PlaceholderMode{
    Default,//mysql,sqlite; the placeholder is ?
    PgSql,//postgresql;  the placeholder is $Number
}

const Q_CHAR:u8 = "?".as_bytes()[0];
const D_CHAR:u8 = "$".as_bytes()[0];
const NUM_0:u8 = "0".as_bytes()[0];
const NUM_9:u8 = "9".as_bytes()[0];

pub fn sql_placeholder_transfer(sql:String,mode:PlaceholderMode) -> String {
    let bytes = sql.as_bytes();

    let mut parts = vec![];
    let mut pre_i=0;
    let len = bytes.len();
    let mut i = 0;
    let mut use_defult=false;
    let mut use_pg=false;
    while i<len {
        let char = bytes[i];
        if char == Q_CHAR {
            parts.push((pre_i,i));
            use_defult=true;
            pre_i = i+1;
        }
        else if char==D_CHAR {
            parts.push((pre_i,i));
            use_pg=true;
            i+=1;
            while i<len {
                let char = bytes[i];
                if char < NUM_0 || char > NUM_9 {
                    break;
                }
                i+=1;
            }
            pre_i = i;
            continue;
        }
        i+=1;
    }
    match mode {
        PlaceholderMode::Default => {
            if use_pg==false {
                return sql;
            }
        },
        PlaceholderMode::PgSql => {
            if use_defult==false {
                return sql;
            }
        },
    }
    let mut r_bytes=vec![];
    for i in 0..parts.len() {
        let (p,q)=parts[i];
        for c in &bytes[p..q]{
            r_bytes.push(*c);
        }
        match mode {
            PlaceholderMode::Default => r_bytes.push(Q_CHAR),
            PlaceholderMode::PgSql => {
                for c in format!("${}",i+1).as_bytes() {
                    r_bytes.push(*c);
                }
            }
        };
    }
    if pre_i < len {
        for c in &bytes[pre_i..len]{
            r_bytes.push(*c);
        }
    }
    String::from_utf8(r_bytes).unwrap()
}


#[derive(Debug,Clone)]
pub struct SqlBuilder<'a> {
    sqls: Vec<InnerSql<'a>>,
    args: Vec<Value>,
    join_str: &'a str,
    prefix_trim: &'a str,
    suffix_trim: &'a str,
    prefix: &'a str,
    suffix: &'a str,
    mode: PlaceholderMode,
}

impl<'a> SqlBuilder<'a> {

    pub fn prepare(builder:&mut SqlBuilder<'a>) -> (String,Vec<serde_json::Value>) {
        builder.build()
    }

    pub fn b(builder:&mut SqlBuilder<'a>) -> (String,Vec<serde_json::Value>) {
        Self::prepare(builder)
    }

    pub fn real(builder:&mut SqlBuilder<'a>) -> Self {
        let mut top = Self::new();
        top.push_build(builder);
        top
    }

    pub fn new() -> Self {
        Self{
            sqls:Default::default(),
            args:Default::default(),
            join_str: " ",
            prefix_trim: "",
            suffix_trim: "",
            prefix: "",
            suffix: "",
            mode: PlaceholderMode::Default,
        }
    }

    pub fn new_builder(join_str:&'a str,trim_str:&'a str,prefix:&'a str,suffix:&'a str) -> Self {
        Self{
            sqls:Default::default(),
            args:Default::default(),
            join_str: join_str,
            prefix_trim: trim_str,
            suffix_trim: trim_str,
            prefix: prefix,
            suffix: suffix,
            mode: PlaceholderMode::Default,
        }
    }

    pub fn new_where() -> Self {
        Self::new_builder(" and ","and","where "," ")
    }

    pub fn new_or() -> Self {
        Self::new_builder(" or ","or","(",")")
    }
    pub fn new_and() -> Self {
        Self::new_builder(" and ","and","(",")")
    }
    pub fn new_comma() -> Self {
        Self::new_builder(" , ",","," "," ")
    }
    pub fn new_comma_paren() -> Self {
        Self::new_builder(" , ",",","(",")")
    }
    pub fn new_paren() -> Self {
        Self::new_builder(" ","","(",")")
    }

    pub fn new_sql(sql:&'a str) -> Self {
        let mut s= Self::new();
        s.push_sql(sql );
        s
    }

    pub fn new_sql_arg<T>(sql:&'a str,arg:&T) -> Self
    where T: serde::ser::Serialize {
        let mut s= Self::new();
        s.push(sql, arg);
        s
    }

    pub fn push<T>(&mut self,sql:&'a str,arg:&T) -> &mut Self 
    where T: serde::ser::Serialize {
        self.sqls.push(InnerSql::Ref(sql));
        self.args.push(json!(arg));
        self
    }

    ///
    /// push &str sql
    /// 
    pub fn push_sql(&mut self,sql:&'a str) -> &mut Self {
        self.sqls.push(InnerSql::Ref(sql));
        self
    }

    ///
    /// push String sql
    /// 
    pub fn push_string(&mut self,sql:String) -> &mut Self {
        self.sqls.push(InnerSql::Value(sql));
        self
    }

    pub fn push_arg<T>(&mut self,arg:&T) -> &mut Self 
    where T: serde::ser::Serialize {
        self.args.push(json!(arg));
        self
    }

    pub fn push_fn<F>(&mut self,f:F) -> &mut Self 
    where
        F: Fn() -> Self + Send
    {
        self.push_build(&mut f())
    }

    pub fn set_mode(mut self,mode:PlaceholderMode) -> Self{
        self.mode=mode;
        self
    }


    fn is_not_trim(&self) -> bool{
        self.prefix=="" &&self.prefix_trim=="" && self.suffix=="" && self.suffix_trim==""
    }

    pub fn is_empty(&self) -> bool {
        self.sqls.is_empty()
    }

    pub fn eq<T>(&mut self,field:&'a str,arg:&T) -> &mut Self 
    where T: serde::ser::Serialize {
        self.sqls.push(InnerSql::Value(format!("{}=?",field)));
        self.args.push(json!(arg));
        self
    }

    pub fn ne<T>(&mut self,field:&'a str,arg:&T) -> &mut Self 
    where T: serde::ser::Serialize {
        self.sqls.push(InnerSql::Value(format!("{}<>?",field)));
        self.args.push(json!(arg));
        self
    }

    pub fn lt<T>(&mut self,field:&'a str,arg:&T) -> &mut Self 
    where T: serde::ser::Serialize {
        self.sqls.push(InnerSql::Value(format!("{}<?",field)));
        self.args.push(json!(arg));
        self
    }

    pub fn le<T>(&mut self,field:&'a str,arg:&T) -> &mut Self 
    where T: serde::ser::Serialize {
        self.sqls.push(InnerSql::Value(format!("{}<=?",field)));
        self.args.push(json!(arg));
        self
    }

    pub fn gt<T>(&mut self,field:&'a str,arg:&T) -> &mut Self 
    where T: serde::ser::Serialize {
        self.sqls.push(InnerSql::Value(format!("{}>?",field)));
        self.args.push(json!(arg));
        self
    }
    pub fn ge<T>(&mut self,field:&'a str,arg:&T) -> &mut Self 
    where T: serde::ser::Serialize {
        self.sqls.push(InnerSql::Value(format!("{}>=?",field)));
        self.args.push(json!(arg));
        self
    }

    pub fn like<T>(&mut self,field:&'a str,arg:&T) -> &mut Self 
    where T: serde::ser::Serialize {
        self.sqls.push(InnerSql::Value(format!("{} like ?",field)));
        self.args.push(json!(arg));
        self
    }
    pub fn not_like<T>(&mut self,field:&'a str,arg:&T) -> &mut Self 
    where T: serde::ser::Serialize {
        self.sqls.push(InnerSql::Value(format!("{} not like ?",field)));
        self.args.push(json!(arg));
        self
    }

    pub fn r#in<T>(&mut self,field:&'a str,args:&[T]) -> &mut Self 
    where T: serde::ser::Serialize {
        let mut in_value = Self::new_comma_paren();
        for arg in args {
            in_value.push("?", arg);
        }
        let (sql,mut args) = in_value.build();
        self.sqls.push(InnerSql::Value(format!("{} in {}",field,sql)));
        self.args.append(&mut args);
        self
    }

    pub fn not_in<T>(&mut self,field:&'a str,args:&[T]) -> &mut Self 
    where T: serde::ser::Serialize {
        let mut in_value = Self::new_comma_paren();
        for arg in args {
            in_value.push("?", arg);
        }
        let (sql,mut args) = in_value.build();
        self.sqls.push(InnerSql::Value(format!("{} not in {}",field,sql)));
        self.args.append(&mut args);
        self
    }

    ///
    /// build sql like:  order by ${field} [desc]
    /// 
    pub fn order_by(&mut self,field:&'a str,desc:bool) -> &mut Self {
        self.sqls.push(InnerSql::Ref(" order by "));
        self.sqls.push(InnerSql::Ref(field));
        if desc {
            self.sqls.push(InnerSql::Ref(" desc "));
        }
        self
    }

    pub fn limit<T>(&mut self,arg:&T) -> &mut Self 
    where T: serde::ser::Serialize {
        self.sqls.push(InnerSql::Ref("limit ?"));
        self.args.push(json!(arg));
        self
    }

    pub fn offset<T>(&mut self,arg:&T) -> &mut Self 
    where T: serde::ser::Serialize {
        self.sqls.push(InnerSql::Ref("offset ?"));
        self.args.push(json!(arg));
        self
    }

    pub fn wrap(&mut self,b:&mut Self) -> &mut Self{
        let (sql,mut args) = b.build();
        self.sqls.push(InnerSql::Value(sql));
        self.args.append(&mut args);
        self
    }

    pub fn push_build(&mut self,b:&mut Self) -> &mut Self{
        if b.join_str==self.join_str && b.is_not_trim() {
            self.sqls.append(&mut b.sqls);
            self.args.append(&mut b.args);
            return self
        }
        self.wrap(b)
    }

    pub fn push_ibuild(&mut self,b:&mut Box<dyn IBuilder>) -> &mut Self{
        let (sql,mut args) = b.build();
        self.sqls.push(InnerSql::Value(sql));
        self.args.append(&mut args);
        self
    }

    fn build_sql(&self) -> String {
        if self.sqls.is_empty() {
            return "".to_owned()
        }
        let sqls=self.sqls.iter().map(|e| {
            match e {
                InnerSql::Value(v) => v,
                InnerSql::Ref(v) => *v,
            }
        }).collect::<Vec<_>>();
        let sql = sqls.join(self.join_str);
        let trim_sql=sql_trim_string(self.prefix_trim, self.suffix_trim, &sql);
        //println!("build_sql:{},{}",&sql,&trim_sql);
        let v=format!("{}{}{}",self.prefix,&trim_sql,self.suffix);
        sql_placeholder_transfer(v, self.mode.clone())
        /*
        match self.mode {
            ArgPlaceholderMode::Default => v,
            ArgPlaceholderMode::PgSql => {
                sql_placeholder_transfer(v, ArgPlaceholderMode::PgSql)
            },
        }
         */
    }
}

impl IBuilder for SqlBuilder<'_> {
    fn build(&mut self) -> (std::string::String, std::vec::Vec<serde_json::Value>) {
        let mut args = vec![];
        args.append(&mut self.args);
        (self.build_sql(),args)
    }
}

impl ToString for SqlBuilder<'_> {
    fn to_string(&self) -> String{
        format!("sql: {} | args:{:?}",&self.build_sql(),self.args)
    }
}

pub use SqlBuilder as B;

#[cfg(test)]
mod tests {
    use crate::sql_placeholder_transfer;

    use super::SqlBuilder;
    use super::B;
    use super::IBuilder;
    use super::sql_trim;
    use super::sql_trim_string;

    #[test]
    fn test_sql(){
        let mut w= SqlBuilder::new();
        w.push_sql("select * from tb_foo");
        w.push("where id= ?",&1).push(" and name = ?",&"test");
        println!("w:{:?}",w);
        println!("build result:{:?}",w.build());
    }

    #[test]
    fn test_wrapper(){
        let mut w= SqlBuilder::new();
        w.push_sql("select * from tb_foo");
        let mut w1= SqlBuilder::new_sql_arg("where id= ?",&1);
        let mut w2 = SqlBuilder::new_sql_arg(" and name = ?",&"test");
        w.wrap(&mut w1);
        w.wrap(&mut w2);
        println!("w:{:?}",w);
        println!("build result:{:?}",w.build());
    }

    #[test]
    fn test_sql_trim(){
        println!("1:'{}'",&sql_trim_string("Where",""," WHERE a=1 "));
        println!("2:'{}'",&sql_trim_string("and",""," and a=1 and b=2 "));
        println!("3:'{}'",&sql_trim_string("and","","  and a=1 and b=2  "));
        println!("4:'{}'",&sql_trim_string(",",",","  a=1 , b=2 , "));
        println!("5:'{}'",&sql_trim_string(",",","," , a=1 , b=2 "));
    }


    #[test]
    fn test_push_build(){
        let mut b = B::new();
        let v= b.push_sql("select * from tb_foo") 
            .push_build(B::new_where()
                .eq("a",&1)
                .eq("b",&"a")
                .lt("c",&3)
                .r#in("d",&[&1,&2,&3])
            ).build();
        println!("{:?}",&v);
    }

    #[test]
    fn test_sql_args(){
        let (sql,args)=B::prepare(
    B::new_sql("select * from tb_foo")
            .push_build(B::new_where()
                .eq("a",&1)
                .eq("b",&"a")
                .lt("c",&3)
                .r#in("d",&[&1,&2,&3])
            )   
        );
        println!("{},{:?}",&sql,&args);
    }

    #[test]
    fn test_default_to_pg_style(){
        let sql ="select id,name,email,age from tb_foo where id in ($1 , $2 , $3) and name in ($4 , $5) and (name=$6 or email=$7) and age=$8 and age>=$9 and age<$10  limit ? offset ?"; 
        let mysql_sql="select id,name,email,age from tb_foo where id in (? , ? , ?) and name in (? , ?) and (name=? or email=?) and age=? and age>=? and age<?  limit ? offset ?";
        let pg_sql="select id,name,email,age from tb_foo where id in ($1 , $2 , $3) and name in ($4 , $5) and (name=$6 or email=$7) and age=$8 and age>=$9 and age<$10  limit $11 offset $12";
        let o_mysql_sql = sql_placeholder_transfer(sql.to_owned(), crate::PlaceholderMode::Default);
        let o_pg_sql = sql_placeholder_transfer(sql.to_owned(), crate::PlaceholderMode::PgSql);
        println!("{}",&o_mysql_sql);
        println!("{}",&o_pg_sql);
        assert_eq!(mysql_sql,&o_mysql_sql);
        assert_eq!(pg_sql,&o_pg_sql);
    }
}