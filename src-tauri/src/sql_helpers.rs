pub(crate) fn push_i64_bind_list(query: &mut sqlx::QueryBuilder<'_, sqlx::Sqlite>, values: &[i64]) {
    let mut separated = query.separated(", ");
    for value in values {
        separated.push_bind(*value);
    }
    separated.push_unseparated(" ");
}

#[cfg(test)]
mod tests {
    use super::push_i64_bind_list;
    use sqlx::QueryBuilder;

    #[tokio::test]
    async fn push_i64_bind_list_binds_values_in_order() {
        let pool = sqlx::SqlitePool::connect(":memory:")
            .await
            .expect("connect in-memory db");
        let mut query = QueryBuilder::new(
            "SELECT value FROM (
                SELECT 1 AS value
                UNION ALL SELECT 2
                UNION ALL SELECT 3
             ) WHERE value IN (",
        );
        push_i64_bind_list(&mut query, &[3, 1]);
        query.push(") ORDER BY value ASC");

        let rows = query
            .build_query_as::<(i64,)>()
            .fetch_all(&pool)
            .await
            .expect("run query");

        assert_eq!(
            rows.into_iter().map(|row| row.0).collect::<Vec<_>>(),
            vec![1, 3]
        );
    }
}
