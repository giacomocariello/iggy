/* Licensed to the Apache Software Foundation (ASF) under one
 * or more contributor license agreements.  See the NOTICE file
 * distributed with this work for additional information
 * regarding copyright ownership.  The ASF licenses this file
 * to you under the Apache License, Version 2.0 (the
 * "License"); you may not use this file except in compliance
 * with the License.  You may obtain a copy of the License at
 *
 *   http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing,
 * software distributed under the License is distributed on an
 * "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
 * KIND, either express or implied.  See the License for the
 * specific language governing permissions and limitations
 * under the License.
 */

use crate::binary::handlers::system::COMPONENT;
use crate::binary::mapper;
use crate::binary::sender::SenderKind;
use crate::streaming::session::Session;
use crate::streaming::systems::system::SharedSystem;
use error_set::ErrContext;
use iggy::error::IggyError;
use iggy::locking::IggySharedMutFn;
use iggy::system::get_me::GetMe;
use tracing::debug;

pub async fn handle(
    command: GetMe,
    sender: &mut SenderKind,
    session: &Session,
    system: &SharedSystem,
) -> Result<(), IggyError> {
    debug!("session: {session}, command: {command}");
    let bytes;
    {
        let system = system.read().await;
        let Some(client) = system
            .get_client(session, session.client_id)
            .await
            .with_error_context(|error| {
                format!("{COMPONENT} (error: {error}) - failed to get current client for session: {session}")
            })?
        else {
            return Err(IggyError::ClientNotFound(session.client_id));
        };

        {
            let client = client.read().await;
            bytes = mapper::map_client(&client);
        }
    }
    sender.send_ok_response(&bytes).await?;
    Ok(())
}
