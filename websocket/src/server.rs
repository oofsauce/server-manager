use crate::{communicator::{self, Communicator}, communicators::{CommunicatorType, csgo::CSGORcon}};
use serde::{Serialize, Deserialize};
use serde_json::Value;
use uuid::Uuid;
use chrono::{Utc};
use std::cmp;

#[derive(Debug)]
#[derive(Serialize, Deserialize)]
#[derive(Clone)]
pub enum CommunicatorStatus {
  DISCONNECTED,
  CONNECTING,
  CONNECTED,
  MISSING
}

/* TODO: separate ServerInfo and ServerSettings 
 * makes more sense to have a dedicated struct for user configurable properties,
 * and to reference this in ServerInfo */
#[derive(Debug)]
#[derive(Serialize, Deserialize)]
#[derive(Clone)]
pub struct ServerInfo { 
  pub id: Uuid,
  pub name: String,
  pub communicator: CommunicatorStatus,
  pub settings: Value,
  pub clients: Value,
}
#[derive(Debug)]
#[derive(Serialize, Deserialize)]
#[derive(Clone)]
pub enum MessageType {
  IN,
  OUT
}

#[derive(Debug)]
#[derive(Serialize, Deserialize)]
#[derive(Clone)]
pub struct Message {
  pub timestamp: i64,
  pub body: String,
  pub msg_type: MessageType,
}

impl Message {
  pub fn new(body: String, msg_type: MessageType) -> Message {
    Message {
      timestamp: Utc::now().timestamp(),
      body,
      msg_type,
    }
  }
}

pub const PAGE_SIZE: usize = 50; // server page size, in messages

type BoxedCommunicator = Box<dyn Communicator + Send + Sync>;

pub struct Server {
  id: Uuid,
  info: ServerInfo,
  communicator: Option<BoxedCommunicator>,
  messages: Vec<Message>,
}

impl Server {
  pub fn new(name: String, communicator: Option<BoxedCommunicator>) -> Server {
    let id = Uuid::new_v4();
    Server {
      id,
      info: ServerInfo {
        id, 
        name,
        communicator: CommunicatorStatus::DISCONNECTED,
        settings: Value::Null,
        clients: Value::Null,
      },
      communicator,
      messages: Vec::new(),
    }
  }
  // TODO: better server creation infra
  // im 90% sure theres a way to delegate the creation code to the communicators themselves
  // that way the footprint of a new communicator is ideally only in 1 file
  pub fn create(name: String, communicator: Option<CommunicatorType>) -> Server {
    match communicator {
      Some(communicator) => {
        match communicator {
          CommunicatorType::CSGO => {
            Server::new(name, Some(Box::new(CSGORcon::new())))
          },
        }
      },
      None => {
        let mut s = Server::new(name, None);
        s.info.communicator = CommunicatorStatus::MISSING;
        s
      }
    }
  }

  pub fn id(&self) -> &Uuid {
    return &self.id;
  } 

  pub async fn send_cmd(&mut self, cmd: String) -> String {
    self.messages.push(Message::new(cmd.clone(), MessageType::IN));
    match self.communicator.as_mut() {
      Some(communicator) => {
        let r = communicator.send_cmd(cmd).await;
        self.messages.push(Message::new(r.clone(), MessageType::OUT));
        r
      },
      None => {
        let s = "No communicator has been set up!".to_string();
        self.messages.push(Message::new(s.clone(), MessageType::OUT));
        s
      }
    }
  }
  
  pub async fn connect(&mut self, address: &str, password: &str) -> Result<(), communicator::Error> {
    match self.communicator.as_mut() {
      Some(communicator) => {
        self.info.communicator = CommunicatorStatus::CONNECTING;
        let res = communicator.connect(address, password).await;
        if res.is_err() {
          self.info.communicator = CommunicatorStatus::DISCONNECTED;
          return Err(communicator::Error::ConnectionError);
        } else {
          self.info.communicator = CommunicatorStatus::CONNECTED;
        }
        Ok(())
      },
      None => {
        self.info.communicator = CommunicatorStatus::MISSING;
        Ok(())
      }
    } 
  }

  pub fn get_page(&self, page_no: usize) -> Vec<Message> {
    let start = page_no*PAGE_SIZE;
    self.messages[start.. cmp::min(start+PAGE_SIZE, self.messages.len())].to_vec()
  }

  pub fn message_count(&self) -> usize {
    self.messages.len()
  }
  
  pub fn info(&self) -> ServerInfo {
    return self.info.clone();
  }

  pub fn get_settings(&self) -> Value {
    match self.communicator.as_ref() {
      Some(comm) => {
        comm.settings()
      },
      None => {
        Value::Null
      }
    }
  }

  fn name(&self) -> &String {
    return &self.info.name;
  }

  fn set_name(&mut self, name: String) {
    self.info.name = name;
  }
}