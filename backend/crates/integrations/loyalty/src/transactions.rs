use diesel::prelude::*;

use crate::db::Pool;
use crate::error::EngineResult;
use crate::models::{PointsTransaction, NewTransaction};
use bigdecimal::BigDecimal;

#[derive(Clone)]
pub struct TransactionService {
    pool: Pool,
}

impl TransactionService {
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }

    pub async fn create(
        &self,
        id: &str,
        member_id: &str,
        program_id: &str,
        source_order: &str,
        delta: i32,
        amount_total: BigDecimal,
    ) -> EngineResult<PointsTransaction> {
        let new = NewTransaction {
            id: id.to_string(),
            member_id: member_id.to_string(),
            program_id: program_id.to_string(),
            source_system: "odoo".to_string(),
            source_order: source_order.to_string(),
            delta,
            amount_total,
        };

        let conn = self.pool.get().await?;
        let member = conn
            .interact(move |conn| {
                use crate::schema::points_transactions::dsl::points_transactions;
                diesel::insert_into(points_transactions)
                    .values(&new)
                    .returning(PointsTransaction::as_returning())
                    .get_result::<PointsTransaction>(conn)
            })
            .await??;
        Ok(member)
    }

}
