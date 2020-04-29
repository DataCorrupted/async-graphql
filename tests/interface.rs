use async_graphql::*;

#[test]
#[should_panic]
pub fn test_interface_field_type_do_not_match() {
    #[SimpleObject]
    struct MyObj {
        id: i32,
    }

    #[Interface(field(name = "id", type = "String"))]
    struct MyInterface(MyObj);

    struct Query;

    #[Object]
    impl Query {
        async fn obj(&self) -> MyInterface {
            unimplemented!()
        }
    }

    Schema::new(Query, EmptyMutation, EmptySubscription);
}

#[test]
#[should_panic]
pub fn test_interface_field_does_not_exists() {
    #[SimpleObject]
    struct MyObj {
        id: i32,
    }

    #[Interface(field(name = "id1", type = "String"))]
    struct MyInterface(MyObj);

    struct Query;

    #[Object]
    impl Query {
        async fn obj(&self) -> MyInterface {
            unimplemented!()
        }
    }

    Schema::new(Query, EmptyMutation, EmptySubscription);
}

#[test]
#[should_panic]
pub fn test_interface_field_number_of_parameters_does_not_match() {
    struct MyObj;

    #[Object]
    impl MyObj {
        #[allow(unused_variables)]
        async fn value(&self, a: i32) -> String {
            unimplemented!()
        }
    }

    #[Interface(field(
        name = "value",
        type = "String",
        arg(name = "a", type = "i32"),
        arg(name = "b", type = "i32")
    ))]
    struct MyInterface(MyObj);

    struct Query;

    #[Object]
    impl Query {
        async fn obj(&self) -> MyInterface {
            unimplemented!()
        }
    }

    Schema::new(Query, EmptyMutation, EmptySubscription);
}

#[test]
#[should_panic]
pub fn test_interface_field_parameter_type_do_not_match() {
    struct MyObj;

    #[Object]
    impl MyObj {
        #[allow(unused_variables)]
        async fn value(&self, a: i32) -> String {
            unimplemented!()
        }
    }

    #[Interface(field(name = "value", type = "String", arg(name = "a", type = "String")))]
    struct MyInterface(MyObj);

    struct Query;

    #[Object]
    impl Query {
        async fn obj(&self) -> MyInterface {
            unimplemented!()
        }
    }

    Schema::new(Query, EmptyMutation, EmptySubscription);
}

#[test]
#[should_panic]
pub fn test_interface_field_parameter_does_not_exists() {
    struct MyObj;

    #[Object]
    impl MyObj {
        #[allow(unused_variables)]
        async fn value(&self, a: i32) -> String {
            unimplemented!()
        }
    }

    #[Interface(field(name = "value", type = "String", arg(name = "b", type = "i32")))]
    struct MyInterface(MyObj);

    struct Query;

    #[Object]
    impl Query {
        async fn obj(&self) -> MyInterface {
            unimplemented!()
        }
    }

    Schema::new(Query, EmptyMutation, EmptySubscription);
}

#[async_std::test]
pub async fn test_interface_simple_object() {
    #[async_graphql::SimpleObject]
    struct MyObj {
        id: i32,
        title: String,
    }

    #[async_graphql::Interface(field(name = "id", type = "i32"))]
    struct Node(MyObj);

    struct Query;

    #[Object]
    impl Query {
        async fn node(&self) -> Node {
            MyObj {
                id: 33,
                title: "haha".to_string(),
            }
            .into()
        }
    }

    let query = r#"{
            node {
                ... on Node {
                    id
                }
            }
        }"#;
    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
    assert_eq!(
        schema.execute(&query).await.unwrap().data,
        serde_json::json!({
            "node": {
                "id": 33,
            }
        })
    );
}

#[async_std::test]
pub async fn test_interface_simple_object2() {
    #[async_graphql::SimpleObject]
    struct MyObj {
        id: i32,
        title: String,
    }

    #[async_graphql::Interface(field(name = "id", type = "&i32"))]
    struct Node(MyObj);

    struct Query;

    #[Object]
    impl Query {
        async fn node(&self) -> Node {
            MyObj {
                id: 33,
                title: "haha".to_string(),
            }
            .into()
        }
    }

    let query = r#"{
            node {
                ... on Node {
                    id
                }
            }
        }"#;
    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
    assert_eq!(
        schema.execute(&query).await.unwrap().data,
        serde_json::json!({
            "node": {
                "id": 33,
            }
        })
    );
}

#[async_std::test]
pub async fn test_multiple_interfaces() {
    struct MyObj;

    #[async_graphql::Object]
    impl MyObj {
        async fn value_a(&self) -> i32 {
            1
        }

        async fn value_b(&self) -> i32 {
            2
        }

        async fn value_c(&self) -> i32 {
            3
        }
    }

    #[async_graphql::Interface(field(name = "value_a", type = "i32"))]
    struct InterfaceA(MyObj);

    #[async_graphql::Interface(field(name = "value_b", type = "i32"))]
    struct InterfaceB(MyObj);

    struct Query;

    #[Object]
    impl Query {
        async fn my_obj(&self) -> InterfaceB {
            MyObj.into()
        }
    }

    let schema = Schema::build(Query, EmptyMutation, EmptySubscription)
        .register_type::<InterfaceA>() // `InterfaceA` is not directly referenced, so manual registration is required.
        .finish();
    let query = r#"{
            myObj {
               ... on InterfaceA {
                valueA
              }
              ... on InterfaceB {
                valueB
              }
              ... on MyObj {
                valueC
              }
            }
        }"#;
    assert_eq!(
        schema.execute(&query).await.unwrap().data,
        serde_json::json!({
            "myObj": {
                "valueA": 1,
                "valueB": 2,
                "valueC": 3,
            }
        })
    );
}

#[async_std::test]
pub async fn test_multiple_objects_in_multiple_interfaces() {
    struct MyObjOne;

    #[async_graphql::Object]
    impl MyObjOne {
        async fn value_a(&self) -> i32 {
            1
        }

        async fn value_b(&self) -> i32 {
            2
        }

        async fn value_c(&self) -> i32 {
            3
        }
    }

    struct MyObjTwo;

    #[async_graphql::Object]
    impl MyObjTwo {
        async fn value_a(&self) -> i32 {
            1
        }
    }

    #[async_graphql::Interface(field(name = "value_a", type = "i32"))]
    struct InterfaceA(MyObjOne, MyObjTwo);

    #[async_graphql::Interface(field(name = "value_b", type = "i32"))]
    struct InterfaceB(MyObjOne);

    struct Query;

    #[Object]
    impl Query {
        async fn my_obj(&self) -> Vec<InterfaceA> {
            vec![MyObjOne.into(), MyObjTwo.into()]
        }
    }

    let schema = Schema::build(Query, EmptyMutation, EmptySubscription)
        .register_type::<InterfaceB>() // `InterfaceB` is not directly referenced, so manual registration is required.
        .finish();
    let query = r#"{
             myObj {
                ... on InterfaceA {
                 valueA
               }
               ... on InterfaceB {
                 valueB
               }
               ... on MyObjOne {
                 valueC
               }
             }
         }"#;
    assert_eq!(
        schema.execute(&query).await.unwrap().data,
        serde_json::json!({
            "myObj": [{
                "valueA": 1,
                "valueB": 2,
                "valueC": 3,
            }, {
                "valueA": 1
            }]
        })
    );
}

#[async_std::test]
pub async fn test_interface_field_result() {
    struct MyObj;

    #[async_graphql::Object]
    impl MyObj {
        async fn value(&self) -> FieldResult<i32> {
            Ok(10)
        }
    }

    #[async_graphql::Interface(field(name = "value", type = "FieldResult<i32>"))]
    struct Node(MyObj);

    struct Query;

    #[Object]
    impl Query {
        async fn node(&self) -> Node {
            MyObj.into()
        }
    }

    let query = r#"{
            node {
                ... on Node {
                    value
                }
            }
        }"#;
    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
    assert_eq!(
        schema.execute(&query).await.unwrap().data,
        serde_json::json!({
            "node": {
                "value": 10,
            }
        })
    );
}
