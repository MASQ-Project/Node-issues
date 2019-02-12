// Copyright (c) 2017-2019, Substratum LLC (https://substratum.net) and/or its affiliates. All rights reserved.
use crate::tokio_wrapper_mocks::ReadHalfWrapperMock;
use crate::tokio_wrapper_mocks::WriteHalfWrapperMock;
use futures::future::result;
use std::cell::RefCell;
use std::io;
use std::net::IpAddr;
use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::Mutex;
use sub_lib::logger::Logger;
use sub_lib::stream_connector::ConnectionInfo;
use sub_lib::stream_connector::ConnectionInfoFuture;
use sub_lib::stream_connector::StreamConnector;
use tokio::net::TcpStream;
use tokio::prelude::Async;

pub struct StreamConnectorMock {
    connect_pair_params: Arc<Mutex<Vec<SocketAddr>>>,
    connect_pair_results: RefCell<Vec<Result<ConnectionInfo, io::Error>>>,
}

impl StreamConnector for StreamConnectorMock {
    fn connect(&self, socket_addr: SocketAddr, _logger: &Logger) -> ConnectionInfoFuture {
        self.connect_pair_params.lock().unwrap().push(socket_addr);
        let connection_info_result = self.connect_pair_results.borrow_mut().remove(0);
        Box::new(result(connection_info_result))
    }

    fn connect_one(
        &self,
        _ip_addrs: Vec<IpAddr>,
        _target_hostname: &String,
        _target_port: u16,
        _logger: &Logger,
    ) -> Result<ConnectionInfo, io::Error> {
        self.connect_pair_results.borrow_mut().remove(0)
    }

    fn split_stream(&self, _stream: TcpStream, _logger: &Logger) -> ConnectionInfo {
        unimplemented!()
    }

    fn split_stream_fut(&self, _stream: TcpStream, _logger: &Logger) -> ConnectionInfoFuture {
        unimplemented!()
    }
}

impl StreamConnectorMock {
    pub fn new() -> StreamConnectorMock {
        Self {
            connect_pair_params: Arc::new(Mutex::new(vec![])),
            connect_pair_results: RefCell::new(vec![]),
        }
    }

    pub fn connection(
        self,
        local_addr: SocketAddr,
        peer_addr: SocketAddr,
        reads: Vec<(Vec<u8>, Result<Async<usize>, io::Error>)>,
        writes: Vec<Result<Async<usize>, io::Error>>,
    ) -> StreamConnectorMock {
        let read_half = reads
            .into_iter()
            .fold(ReadHalfWrapperMock::new(), |so_far, elem| {
                so_far.poll_read_result(elem.0, elem.1)
            });
        let write_half = writes
            .into_iter()
            .fold(WriteHalfWrapperMock::new(), |so_far, elem| {
                so_far.poll_write_result(elem)
            });
        let connection_info = ConnectionInfo {
            reader: Box::new(read_half),
            writer: Box::new(write_half),
            local_addr,
            peer_addr,
        };
        self.connect_pair_result(Ok(connection_info))
    }

    pub fn with_connection(
        self,
        local_addr: SocketAddr,
        peer_addr: SocketAddr,
        reader: ReadHalfWrapperMock,
        writer: WriteHalfWrapperMock,
    ) -> StreamConnectorMock {
        let connection_info = ConnectionInfo {
            reader: Box::new(reader),
            writer: Box::new(writer),
            local_addr,
            peer_addr,
        };
        self.connect_pair_result(Ok(connection_info))
    }

    pub fn connect_pair_params(
        mut self,
        params_arc: &Arc<Mutex<Vec<SocketAddr>>>,
    ) -> StreamConnectorMock {
        self.connect_pair_params = params_arc.clone();
        self
    }

    pub fn connect_pair_result(
        self,
        result: Result<ConnectionInfo, io::Error>,
    ) -> StreamConnectorMock {
        self.connect_pair_results.borrow_mut().push(result);
        self
    }
}
