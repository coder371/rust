pub struct Order {
    pub id: u32,
    pub product_id: u32,
    pub quantity: u32,
}
pub fn find_all_orders() -> Vec<Order> {
    vec![
        Order{
            id: 1,
            product_id: 101,
            quantity: 2,
        },
        Order{
            id: 2,
            product_id: 102,
            quantity: 1,
        },
    ]
}