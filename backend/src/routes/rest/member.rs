use crate::database::DatabasePool;
use crate::util::status_json::StatusJson as SJ;
use diesel::prelude::*;
use laggit_api::book_account::{BookAccountId, BookAccountType};
use laggit_api::member::{Member, MemberId, NewMember};
use rocket::{get, post, State};
use rocket_contrib::json::Json;

#[get("/members")]
pub fn get_members(db_pool: State<DatabasePool>) -> Result<Json<Vec<Member>>, SJ> {
    let connection = db_pool.inner().get()?;
    use crate::schema::tables::members::dsl::*;

    Ok(Json(members.load(&connection)?))
}

#[post("/add_member_with_book_account", data = "<data>")]
pub fn add_member_with_book_account(
    db_pool: State<DatabasePool>,
    data: Json<(NewMember, String)>,
) -> Result<Json<(MemberId, BookAccountId)>, SJ> {
    let connection = db_pool.inner().get()?;

    let (new_member, account_name) = data.into_inner();

    connection.transaction::<_, SJ, _>(|| {
        let member_id = {
            use crate::schema::tables::members::dsl::*;

            diesel::insert_into(members)
                .values((
                    first_name.eq(&new_member.first_name),
                    last_name.eq(&new_member.last_name),
                    nickname.eq(&new_member.nickname),
                ))
                .returning(id)
                .get_result(&connection)?
        };

        let acc_id = {
            use crate::schema::tables::book_accounts::dsl::*;

            diesel::insert_into(book_accounts)
                .values((
                    name.eq(&account_name),
                    account_type.eq(&BookAccountType::Liabilities),
                    creditor.eq(&Some(member_id)),
                ))
                .returning(id)
                .get_result(&connection)?
        };

        Ok(Json((member_id, acc_id)))
    })
}
