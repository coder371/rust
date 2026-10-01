use async_graphql::MergedObject;

mod my_orders;
mod order_customer;

#[derive(MergedObject, Default)]
#[graphql(name = "Query")]
pub struct PartnerQuery(my_orders::MyOrdersQuery, order_customer::OrderCustomerQuery);
