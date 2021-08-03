

use serde_json::{json, Value};


#[derive(Debug,Clone)]
enum InnerSql<'a>{
    V(String),
    Ref(&'a str)
}

#[derive(Debug,Clone)]
pub struct SqlBuilder<'a> {
    sqls: Vec<InnerSql<'a>>,
    args: Vec<Value>,
}

impl<'a> SqlBuilder<'a> {
    pub fn new() -> Self {
        Self{
            sqls:Default::default(),
            args:Default::default(),
        }
    }

    pub fn new_sql<T>(sql:&'a str) -> Self{
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

    pub fn push_wrapper(&mut self,mut w:Self) -> &mut Self{
        self.sqls.append(&mut w.sqls);
        self.args.append(&mut w.args);
        self
    }

    fn build_sql(&self) -> String {
        let sqls=self.sqls.iter().map(|e| {
            match e {
                InnerSql::V(v) => v,
                InnerSql::Ref(v) => *v,
            }
        }).collect::<Vec<_>>();
        sqls.join(" ")
    }

    pub fn build(& self)  -> (String,&Vec<Value>) {
        (self.build_sql(),&self.args)
    }
}

#[cfg(test)]
mod tests {
    use super::SqlBuilder;

    #[test]
    fn test_wrapper(){
        let mut w= SqlBuilder::new();
        w.push_sql("select * from tb_foo");
        w.push("where id= ?",&1).push(" and name = ?",&"test");
        println!("w:{:?}",w);
        println!("build result:{:?}",w.build());
    }

    #[test]
    fn test_wrapper02(){
        let mut w= SqlBuilder::new();
        w.push_sql("select * from tb_foo");
        let w1= SqlBuilder::new_sql_arg("where id= ?",&1);
        let w2 = SqlBuilder::new_sql_arg(" and name = ?",&"test");
        w.push_wrapper(w1);
        w.push_wrapper(w2);
        println!("w:{:?}",w);
        println!("build result:{:?}",w.build());
    }
}