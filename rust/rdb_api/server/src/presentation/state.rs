use std::sync::Arc;

use crate::use_case::order::create_order::CreateOrderUseCase;
use crate::use_case::user::create_user::CreateUserUseCase;
use crate::use_case::user::get_user::GetUserUseCase;
use crate::use_case::user::list_users::ListUsersUseCase;

#[derive(Clone)]
pub struct AppState {
    pub create_user: Arc<dyn CreateUserUseCase>,
    pub get_user: Arc<dyn GetUserUseCase>,
    pub list_users: Arc<dyn ListUsersUseCase>,
    pub create_order: Arc<dyn CreateOrderUseCase>,
}
