use anyhow::Result;
use schemas::registry::RegistryStatus;
use crate::store::RegistryStore;

impl RegistryStore {
    pub fn verify_integrity(&self) -> Result<RegistryStatus> {
        let meta = self.check_integrity()?;
        Ok(meta.status)
    }

    pub fn rebuild_indexes(&self) -> Result<()> {
        self.conn.execute_batch("
            DELETE FROM search_index;
            INSERT INTO search_index (term, document_id, frequency)
            SELECT DISTINCT term, document_id, COUNT(*) as frequency
            FROM (
                SELECT LOWER(value) as term, documents.id as document_id
                FROM documents, json_each('[\"placeholder\"]')
                WHERE documents.body != ''
            )
            GROUP BY term, document_id;
        ")?;
        Ok(())
    }
}
