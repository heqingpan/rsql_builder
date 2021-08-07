

use serde_json::{json, Value};

pub trait IBuilder {
    fn build(&mut self)  -> (String,&mut Vec<Value>);
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
pub struct SqlBuilder<'a> {
    sqls: Vec<InnerSql<'a>>,
    args: Vec<Value>,
    join_str: &'a str,
    prefix_trim: &'a str,
    suffix_trim: &'a str,
    prefix: &'a str,
    suffix: &'a str,
}

impl<'a> SqlBuilder<'a> {

    pub fn sql_args(builder:&mut SqlBuilder<'a>) -> (String,Vec<serde_json::Value>) {
        let mut top = Self::new();
        top.push_build(builder);
        (top.build_sql(),top.args)
    }

    pub fn b(builder:&mut SqlBuilder<'a>) -> (String,Vec<serde_json::Value>) {
        Self::sql_args(builder)
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
            suffix: prefix,
        }
    }
    pub fn new_where() -> Self {
        Self::new_builder(" and ","and"," where "," ")
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

    pub fn push_sql(&mut self,sql:&'a str) -> &mut Self {
        self.sqls.push(InnerSql::Ref(sql));
        self
    }

    pub fn push_arg<T>(&mut self,arg:&T) -> &mut Self 
    where T: serde::ser::Serialize {
        self.args.push(json!(arg));
        self
    }

    fn is_not_trim(&self) -> bool{
        self.prefix=="" &&self.prefix_trim=="" && self.suffix=="" && self.suffix_trim==""
    }

    pub fn eq<T>(&mut self,field:&'a str,arg:&T) -> &mut Self 
    where T: serde::ser::Serialize {
        self.sqls.push(InnerSql::Value(format!("{}=?",field)));
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

    pub fn r#in<T>(&mut self,field:&'a str,args:&[T]) -> &mut Self 
    where T: serde::ser::Serialize {
        let mut in_value = Self::new_comma_paren();
        for arg in args {
            in_value.push("?", arg);
        }
        let (sql,args) = in_value.build();
        self.sqls.push(InnerSql::Value(format!("{} in {}",field,sql)));
        self.args.append(args);
        self
    }

    pub fn not_in<T>(&mut self,field:&'a str,args:&[T]) -> &mut Self 
    where T: serde::ser::Serialize {
        let mut in_value = Self::new_comma_paren();
        for arg in args {
            in_value.push("?", arg);
        }
        let (sql,args) = in_value.build();
        self.sqls.push(InnerSql::Value(format!("{} not in {}",field,sql)));
        self.args.append(args);
        self
    }

    pub fn wrap(&mut self,b:&mut Self) -> &mut Self{
        let (sql,args) = b.build();
        self.sqls.push(InnerSql::Value(sql));
        self.args.append(args);
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
        let (sql,args) = b.build();
        self.sqls.push(InnerSql::Value(sql));
        self.args.append(args);
        self
    }

    fn build_sql(&self) -> String {
        let sqls=self.sqls.iter().map(|e| {
            match e {
                InnerSql::Value(v) => v,
                InnerSql::Ref(v) => *v,
            }
        }).collect::<Vec<_>>();
        let sql = sqls.join(self.join_str);
        sql_trim_string(self.prefix_trim, self.suffix_trim, &sql)
        //format!("{}{}{}",self.prefix,&sql_trim_string(self.prefix_trim, self.suffix_trim, &sql),self.suffix)
    }
}

impl IBuilder for SqlBuilder<'_> {
    fn build(&mut self) -> (std::string::String, &mut std::vec::Vec<serde_json::Value>) {
        (self.build_sql(),&mut self.args)
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
        let (sql,args)=B::sql_args(
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
}