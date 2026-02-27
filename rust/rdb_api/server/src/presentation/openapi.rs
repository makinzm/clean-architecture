use utoipa::OpenApi;

use crate::domain::entity::order::Order;
use crate::domain::entity::user::User;
use crate::presentation::handler::order::create_order::CreateOrderRequest;
use crate::presentation::handler::user::create_user::CreateUserRequest;

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::presentation::handler::user::list_users::list_users,
        crate::presentation::handler::user::create_user::create_user,
        crate::presentation::handler::user::get_user::get_user,
        crate::presentation::handler::order::create_order::create_order,
    ),
    components(schemas(
        User,
        Order,
        CreateUserRequest,
        CreateOrderRequest,
    ))
)]
pub struct ApiDoc;
