use async_trait::async_trait;
use sentio_email::{EmailAddress, EmailResult, MessageId, OutgoingMessage, SmtpClient};
use std::sync::Mutex;

pub struct MockSmtpClient {
    pub send_message_calls: Mutex<Vec<OutgoingMessage>>,
    pub connect_calls: Mutex<u32>,
    pub disconnect_calls: Mutex<u32>,
    pub is_connected_value: Mutex<bool>,
}

impl MockSmtpClient {
    pub fn new() -> Self {
        Self {
            send_message_calls: Mutex::new(Vec::new()),
            connect_calls: Mutex::new(0),
            disconnect_calls: Mutex::new(0),
            is_connected_value: Mutex::new(false),
        }
    }
}

#[async_trait]
impl SmtpClient for MockSmtpClient {
    async fn send_message(&self, message: &OutgoingMessage) -> EmailResult<MessageId> {
        self.send_message_calls
            .lock()
            .unwrap()
            .push(message.clone());
        Ok(MessageId::new("mock_message_id".to_string()))
    }

    async fn verify_address(&self, _address: &EmailAddress) -> EmailResult<bool> {
        Ok(true)
    }

    async fn connect(&mut self) -> EmailResult<()> {
        *self.connect_calls.lock().unwrap() += 1;
        *self.is_connected_value.lock().unwrap() = true;
        Ok(())
    }

    async fn disconnect(&mut self) -> EmailResult<()> {
        *self.disconnect_calls.lock().unwrap() += 1;
        *self.is_connected_value.lock().unwrap() = false;
        Ok(())
    }

    async fn is_connected(&self) -> bool {
        *self.is_connected_value.lock().unwrap()
    }
}
