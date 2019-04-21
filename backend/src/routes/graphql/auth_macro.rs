#[macro_export]
macro_rules! gql_auth {
    ($executor:expr, $scope:expr) => {{
        use crate::auth::*;
        $executor.context().get_auth($scope)
    }};
}
