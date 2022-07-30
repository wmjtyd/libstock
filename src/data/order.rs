//! The order-related operations.

use crypto_message::Order;

#[derive(PartialEq, Eq)]
pub enum OrderType {
    Ask,
    Bid,
}

// WIP: refactor needed!
pub fn get_orders<'a>(new: &'a [Order], old: &'a [Order], order_type: OrderType) -> Vec<Order> {
    let mut new_index: usize = 0;
    let mut old_index: usize = 0;

    let mut result = Vec::new();
    let mut is_new_remaining = new_index < new.len();
    let mut is_old_remaining = old_index < old.len();
    while is_new_remaining && is_old_remaining {
        let latest_order = &new[new_index];
        let old_order = &old[old_index];
        if latest_order.price == old_order.price
            && latest_order.quantity_quote == old_order.quantity_quote
        {
            old_index += 1;
            new_index += 1;
        } else {
            match (
                latest_order.price == old_order.price,
                latest_order.quantity_quote == old_order.quantity_quote,
            ) {
                (true, false) => {
                    let updated = Order {
                        price: old_order.price,
                        quantity_base: latest_order.quantity_base,
                        quantity_quote: latest_order.quantity_quote,
                        quantity_contract: latest_order.quantity_contract,
                    };
                    result.push(updated);
                    old_index += 1;
                    new_index += 1;
                }
                (false, false) => {
                    let mut cross_over = latest_order.price < old_order.price;
                    if order_type == OrderType::Bid {
                        cross_over = latest_order.price > old_order.price;
                    };
                    match cross_over {
                        true => {
                            let removed = Order {
                                price: old_order.price,
                                quantity_base: 0.0,
                                quantity_quote: 0.0,
                                quantity_contract: Some(0.0),
                            };
                            result.push(removed);
                            old_index += 1;
                        }
                        false => {
                            let added = Order {
                                price: old_order.price,
                                quantity_base: 0.0,
                                quantity_quote: 0.0,
                                quantity_contract: Some(0.0),
                            };
                            result.push(added);
                            new_index += 1;
                        }
                    }
                }
                (_, _) => {}
            }
        }
        is_new_remaining = new_index < new.len();
        is_old_remaining = old_index < old.len();
    }
    if is_new_remaining {
        (new_index..new.len()).for_each(|i| {
            let order = &new[i];
            let added = Order {
                price: order.price,
                quantity_base: order.quantity_base,
                quantity_quote: order.quantity_quote,
                quantity_contract: order.quantity_contract,
            };
            result.push(added);
        });
    } else if is_old_remaining {
        (old_index..old.len()).for_each(|i| {
            let order = &old[i];
            let removed = Order {
                price: order.price,
                quantity_base: 0.0,
                quantity_quote: 0.0,
                quantity_contract: None,
            };
            result.push(removed);
        });
    }
    result
}

pub fn restore_orders<'a>(old: &'a [Order], diff: &'a [Order], _type: OrderType) -> Vec<Order> {
    let mut result = Vec::new();
    if diff.is_empty() {
        return result;
    }
    let mut diff_index: usize = 0;
    old.iter().for_each(|order| {
        let diff_order = &diff[diff_index];
        let mut is_coss_over = order.price > diff_order.price;
        if _type == OrderType::Bid {
            is_coss_over = order.price < diff_order.price;
        };
        match (
            order.price == diff_order.price,
            is_coss_over,
            diff_order.quantity_quote,
        ) {
            (true, true, quote) if quote == 0.0 => {}
            (true, _, _) => {
                let updated = Order {
                    price: diff_order.price,
                    quantity_base: diff_order.quantity_base,
                    quantity_quote: diff_order.quantity_quote,
                    quantity_contract: diff_order.quantity_contract,
                };
                result.push(updated);
            }
            (false, true, _) => {
                let old = Order {
                    price: diff_order.price,
                    quantity_base: diff_order.quantity_base,
                    quantity_quote: diff_order.quantity_quote,
                    quantity_contract: diff_order.quantity_contract,
                };
                result.push(old);
            }
            (false, false, _) => {
                let added = Order {
                    price: order.price,
                    quantity_base: order.quantity_base,
                    quantity_quote: order.quantity_quote,
                    quantity_contract: order.quantity_contract,
                };
                let old = Order {
                    price: diff_order.price,
                    quantity_base: diff_order.quantity_base,
                    quantity_quote: diff_order.quantity_quote,
                    quantity_contract: diff_order.quantity_contract,
                };
                diff_index += 1;
                result.push(added);
                result.push(old);
            }
        }
    });
    // dbg!(&result);
    if diff_index < diff.len() {
        (diff_index..diff.len()).for_each(|i| {
            let order = &diff[i];
            let added = Order {
                price: order.price,
                quantity_base: order.quantity_base,
                quantity_quote: order.quantity_quote,
                quantity_contract: order.quantity_contract,
            };
            result.push(added);
        });
    }
    result
}
