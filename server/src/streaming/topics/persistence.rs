use crate::state::system::TopicState;
use crate::streaming::topics::topic::Topic;
use crate::streaming::topics::COMPONENT;
use error_set::ErrContext;
use iggy::error::IggyError;
use iggy::locking::IggySharedMutFn;

impl Topic {
    pub async fn load(&mut self, state: TopicState) -> Result<(), IggyError> {
        let storage = self.storage.clone();
        storage.topic.load(self, state).await?;
        Ok(())
    }

    pub async fn persist(&self) -> Result<(), IggyError> {
        self.storage.topic.save(self).await
    }

    pub async fn delete(&self) -> Result<(), IggyError> {
        for partition in self.get_partitions() {
            let mut partition = partition.write().await;
            partition.delete().await.with_error_context(|error| {
                format!(
                    "{COMPONENT} (error: {error}) - failed to delete partition with ID: {} in topic with ID: {}",
                    partition.partition_id, self.topic_id
                )
            })?;
        }

        self.storage.topic.delete(self).await
    }

    pub async fn persist_messages(&self) -> Result<usize, IggyError> {
        let mut saved_messages_number = 0;
        for partition in self.get_partitions() {
            let mut partition = partition.write().await;
            let partition_id = partition.partition_id;
            for segment in partition.get_segments_mut() {
                saved_messages_number += segment.persist_messages(None).await.with_error_context(|error| format!("{COMPONENT} (error: {error}) - failed to persist messages in segment, partition ID: {partition_id}"))?;
            }
        }

        Ok(saved_messages_number)
    }

    pub async fn purge(&self) -> Result<(), IggyError> {
        for partition in self.get_partitions() {
            let mut partition = partition.write().await;
            partition.purge().await?;
        }
        Ok(())
    }
}
