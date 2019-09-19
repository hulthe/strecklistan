use crate::database::DatabasePool;
use crate::models::book_account as relational;
use crate::models::transaction::relational::Transaction;
use crate::util::status_json::StatusJson as SJ;
use diesel::prelude::*;
use laggit_api::book_account::{BookAccount, BookAccountType, MasterAccounts, NewBookAccount};
use rocket::{get, post, State};
use rocket_contrib::json::Json;
use std::collections::HashMap;

#[get("/book_accounts")]
pub fn get_accounts(db_pool: State<DatabasePool>) -> Result<Json<Vec<BookAccount>>, SJ> {
    let connection = db_pool.inner().get()?;

    let (transactions, accounts) = connection
        .transaction::<(Vec<Transaction>, Vec<relational::BookAccount>), SJ, _>(|| {
            use crate::schema::tables::book_accounts::dsl::book_accounts;
            use crate::schema::tables::transactions::dsl::transactions;
            Ok((
                transactions.load(&connection)?,
                book_accounts.load(&connection)?,
            ))
        })?;

    let mut accounts: HashMap<_, BookAccount> = accounts
        .into_iter()
        .map(|acc| (acc.id, acc.into()))
        .collect();

    for tr in transactions.iter() {
        accounts
            .get_mut(&tr.credited_account)
            .map(|acc| acc.credit(tr.amount.into()));
        accounts
            .get_mut(&tr.debited_account)
            .map(|acc| acc.debit(tr.amount.into()));
    }

    Ok(Json(accounts.into_iter().map(|(_, acc)| acc).collect()))
}

#[post("/book_account", data = "<account>")]
pub fn add_account(
    db_pool: State<DatabasePool>,
    account: Json<NewBookAccount>) -> Result<Json<i32>, SJ> {
    let connection = db_pool.inner().get()?;

    use crate::schema::tables::book_accounts::dsl::*;

    Ok(Json(diesel::insert_into(book_accounts)
        .values((name.eq(&account.name),
                 account_type.eq(&account.account_type),
                 creditor.eq(&account.creditor)))
        .returning(id)
        .get_result(&connection)?))
}

#[get("/book_accounts/masters")]
pub fn get_master_accounts(db_pool: State<DatabasePool>) -> Result<Json<MasterAccounts>, SJ> {
    let connection = db_pool.inner().get()?;
    use crate::schema::tables::book_accounts::dsl::*;

    // TODO: Get the values for the master accounts from some configuration.
    let bank_account_name = "Bankkonto";
    let cash_account_name = "Kontantkassa";
    let sales_account_name = "Försäljning";
    let purchases_account_name = "Inköp";

    connection.transaction::<_, SJ, _>(|| {
        // Make sure the accounts exist in the database
        diesel::insert_into(book_accounts)
            .values((
                name.eq(bank_account_name),
                account_type.eq(BookAccountType::Assets),
            ))
            .on_conflict_do_nothing()
            .execute(&connection)?;
        diesel::insert_into(book_accounts)
            .values((
                name.eq(cash_account_name),
                account_type.eq(BookAccountType::Assets),
            ))
            .on_conflict_do_nothing()
            .execute(&connection)?;
        diesel::insert_into(book_accounts)
            .values((
                name.eq(sales_account_name),
                account_type.eq(BookAccountType::Revenue),
            ))
            .on_conflict_do_nothing()
            .execute(&connection)?;
        diesel::insert_into(book_accounts)
            .values((
                name.eq(purchases_account_name),
                account_type.eq(BookAccountType::Expenses),
            ))
            .on_conflict_do_nothing()
            .execute(&connection)?;

        Ok(Json(MasterAccounts {
            bank_account_id: book_accounts
                .filter(name.eq(bank_account_name))
                .select(id)
                .get_result(&connection)?,
            cash_account_id: book_accounts
                .filter(name.eq(cash_account_name))
                .select(id)
                .get_result(&connection)?,
            sales_account_id: book_accounts
                .filter(name.eq(sales_account_name))
                .select(id)
                .get_result(&connection)?,
            purchases_account_id: book_accounts
                .filter(name.eq(purchases_account_name))
                .select(id)
                .get_result(&connection)?,
        }))
    })
}
