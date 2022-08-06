use rsql_builder::{B, PlaceholderMode};

/* 
example table: 
create table if not exists tb_foo (
    id integer primary key autoincrement,
    name varchar(255),
    email varchar(255),
    age varchar(255),
);
*/

#[derive(Debug,Default)]
pub struct Foo{
    pub id:Option<i64>,
    pub name:Option<String>,
    pub email:Option<String>,
    pub age:Option<i64>,
}

#[derive(Debug,Default)]
pub struct FooParam{
    pub id:Option<i64>,
    pub id_list:Option<Vec<i64>>,
    pub name:Option<String>,
    pub name_list:Option<Vec<String>>,
    pub name_or_email:Option<String>,
    pub age:Option<i64>,
    pub age_begin:Option<i64>,
    pub age_end:Option<i64>,
    pub limit:Option<i64>,
    pub offset:Option<i64>,
}

struct FooInnerDao {
    //connection
}

impl FooInnerDao {

    fn conditions(&self,param:&FooParam) -> B {
        let mut whr = B::new_where()
            .set_mode(PlaceholderMode::PgSql);
        if let Some(id)=&param.id {
            whr.eq("id",id);
        }
        if let Some(id_list)=&param.id_list {
            whr.r#in("id", id_list);
        }
        if let Some(name)=&param.name{
            whr.eq("name", name);
        }
        if let Some(name_list)=&param.name_list{
            whr.r#in("name", name_list);
        }
        if let Some(name_or_email) = &param.name_or_email {
            whr.wrap(B::new_or()
                .eq("name",name_or_email)
                .eq("email",name_or_email)
            );
        }
        if let Some(age) = &param.age {
            whr.eq("age",age);
        }
        if let Some(age_begin) = &param.age_begin {
            whr.ge("age",age_begin);
        }
        if let Some(age_end) = &param.age_end {
            whr.lt("age",age_end);
        }
        whr
    }

    pub fn query_prepare(&self,param:&FooParam) -> (String,Vec<serde_json::Value>) {
        B::prepare(
     B::new_sql("select id,name,email,age from tb_foo")
            //.set_mode(PlaceholderMode::PgSql)
            .push_build(&mut self.conditions(param))
            .push_fn(||{
                let mut b= B::new();
                if let Some(limit) = &param.limit{
                    b.limit(limit);
                }
                if let Some(offset ) = &param.offset{
                    b.offset(offset);
                }
                b
            })
        )
    }

    pub fn insert_prepare(&self,foo:&Foo) -> (String,Vec<serde_json::Value>) {
        let mut field_builder=B::new_comma_paren();
        let mut value_builder=B::new_comma_paren();
        if let Some(id) = &foo.id {
            field_builder.push_sql("id");
            value_builder.push("?",id);
        }
        if let Some(name) = &foo.name {
            field_builder.push_sql("name");
            value_builder.push("?",name);
        }
        if let Some(email) = &foo.email {
            field_builder.push_sql("email");
            value_builder.push("?",email);
        }
        if let Some(age) = &foo.age {
            field_builder.push_sql("age");
            value_builder.push("?",age);
        }
        B::prepare(
     B::new_sql("insert into tb_foo")
            .set_mode(PlaceholderMode::PgSql)
            .push_build(&mut field_builder)
            .push_sql("values")
            .push_build(&mut value_builder)
        )
    }

    pub fn update_prepare(&self,foo:&Foo) -> (String,Vec<serde_json::Value>) {
        let mut set_builder=B::new_comma();
        if let Some(name) = &foo.name {
            //set_builder.push("name=?",name);
            set_builder.eq("name",name);
        }
        if let Some(email) = &foo.email {
            set_builder.push("email=?",email);
        }
        if let Some(age) = &foo.age {
            set_builder.push("age=?",age);
        }
        let mut whr = B::new_where();
        if let Some(id)=&foo.id {
            whr.eq("id",id);
        }
        if whr.is_empty() {
            panic!("update conditions is empty");
        }
        B::prepare(
     B::new_sql("update tb_foo set ")
            .set_mode(PlaceholderMode::PgSql)
            .push_build(&mut set_builder)
            .push_build(&mut whr)
        )
    }

    pub fn delete_prepare(&self,param:&FooParam) -> (String,Vec<serde_json::Value>) {
        B::prepare(
     B::new_sql("delete from tb_foo")
            .push_build(&mut self.conditions(param))
        )
    }

}

fn query_exp(){
    let foo_dao = FooInnerDao{};
    let mut param = FooParam::default();
    let (sql,args)= foo_dao.query_prepare(&param);
    println!("query 01:\n\t'{}'\n\t{:?}",&sql,&args); 

    let mut param = FooParam::default();
    param.id=Some(1);
    let (sql,args)= foo_dao.query_prepare(&param);
    println!("query 02:\n\t'{}'\n\t{:?}",&sql,&args); 

    let mut param = FooParam::default();
    param.id_list = Some(vec![1,2,3]);
    let (sql,args)= foo_dao.query_prepare(&param);
    println!("query 03:\n\t'{}'\n\t{:?}",&sql,&args); 

    let mut param = FooParam::default();
    param.id_list = Some(vec![1,2,3]);
    param.name_list=Some(vec!["foo".to_owned(),"boo".to_owned()]);
    param.name_or_email=Some("foo@foo.com".to_owned());
    param.age=Some(18);
    param.age_begin=Some(16);
    param.age_end=Some(24);
    param.limit=Some(10);
    param.offset=Some(10);
    let (sql,args)= foo_dao.query_prepare(&param);
    println!("query 04:\n\t'{}'\n\t{:?}",&sql,&args); 
}

fn insert_exp(){
    let foo_dao = FooInnerDao{};
    let mut foo = Foo::default();
    foo.id=Some(1);
    foo.name = Some("foo".to_owned());
    let (sql,args)= foo_dao.insert_prepare(&foo);
    println!("insert 01:\n\t'{}'\n\t{:?}",&sql,&args); 

    let mut foo = Foo::default();
    foo.name = Some("foo".to_owned());
    foo.email= Some("foo@foo.com".to_owned());
    let (sql,args)= foo_dao.insert_prepare(&foo);
    println!("insert 02:\n\t'{}'\n\t{:?}",&sql,&args); 

    let mut foo = Foo::default();
    foo.id=Some(3);
    foo.name = Some("foo".to_owned());
    foo.email= Some("foo@foo.com".to_owned());
    foo.age = Some(16);
    let (sql,args)= foo_dao.insert_prepare(&foo);
    println!("insert 03:\n\t'{}'\n\t{:?}",&sql,&args); 
}

fn update_exp(){
    let foo_dao = FooInnerDao{};
    let mut foo = Foo::default();
    foo.id=Some(1);
    foo.name = Some("foo".to_owned());
    let (sql,args)= foo_dao.update_prepare(&foo);
    println!("update 01:\n\t'{}'\n\t{:?}",&sql,&args); 

    let mut foo = Foo::default();
    foo.id=Some(3);
    foo.name = Some("foo".to_owned());
    foo.email= Some("foo@foo.com".to_owned());
    foo.age = Some(16);
    let (sql,args)= foo_dao.update_prepare(&foo);
    println!("update 02:\n\t'{}'\n\t{:?}",&sql,&args); 
}

fn delete_exp(){
    let foo_dao = FooInnerDao{};
    let mut param = FooParam::default();
    param.id_list = Some(vec![1,2,3]);
    param.name_list=Some(vec!["foo".to_owned(),"boo".to_owned()]);
    param.name_or_email=Some("foo@foo.com".to_owned());
    param.age=Some(18);
    param.age_begin=Some(16);
    param.age_end=Some(24);
    let (sql,args)= foo_dao.delete_prepare(&param);
    println!("delete 01:\n\t'{}'\n\t{:?}",&sql,&args); 
}


fn main(){
    let a="012345".as_bytes();
    println!("{:?}",&a[0..0]);
    query_exp();
    insert_exp();
    update_exp();
    delete_exp();
}