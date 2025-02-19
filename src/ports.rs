#[derive(Default)]
/// Input port.
///
/// Used to receive data of type `T`.
pub struct Input<T> {
    rx: Vec<std::sync::mpsc::Receiver<T>>,
}

impl<T> Input<T> {
    pub fn fetch(&mut self) -> Vec<T> {
        let mut ret = Vec::new();
        for r in &mut self.rx {
            'read_empty: loop {
                match r.try_recv() {
                    Ok(data) => ret.push(data),
                    Err(_) => break 'read_empty,
                }
            }
        }
        ret
    }
}

#[derive(Default)]
/// Output port.
/// 
/// Used to send data of type `T`.
pub struct Output<T: Clone> {
    tx: Vec<std::sync::mpsc::Sender<T>>,
}

impl<T: Clone> Output<T> {
    /// Connect this output to a compatible input source.
    /// 
    /// It will send its data to the specified input port.
    pub fn connect(&mut self, input: &mut Input<T>) {
        let (tx, rx) = std::sync::mpsc::channel();
        self.tx.push(tx);
        input.rx.push(rx);
    }

    /// Write data to this port.
    pub fn fire(&mut self, t: T) {
        for tx in &mut self.tx {
            tx.send(t.clone()).expect("Cannot send message");
        }
    }
}
