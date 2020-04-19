pub struct IdGenerator {
    counter: usize,
    stack: Vec<usize>
}

impl IdGenerator {

    pub fn new() -> IdGenerator {
        IdGenerator {
            counter: 1,
            stack: Vec::new()
        }
    }

    pub fn next(&mut self) -> usize {
        if let Some(id) = self.stack.pop() {
            id
        } else {
            self.counter += 1;
            self.counter - 1
        }
    }

    pub fn recycle(&mut self, id: usize) {
        self.stack.push(id)
    }

}
