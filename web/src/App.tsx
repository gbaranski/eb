import React, { useState } from 'react';
import logo from './logo.svg';
import './App.css';

type ServerFrameType = 'update';

interface ServerFrame {
  type: ServerFrameType
};

interface UpdateFrame extends ServerFrame {
  content: string,
}

//

interface ClientFrame {
  type: 'insert' | 'open',
};

interface InsertFrame extends ClientFrame {
  char: string,
  index: number,
}

interface Open extends ClientFrame {
  url: string,
}

const ws = new WebSocket("ws://localhost:8080")

const send = (frame: ClientFrame) => {
  const json = JSON.stringify(frame);
  ws.send(json)
}

const insert = (char: string, index: number) => {
  console.assert(char.length === 1);
  const frame: InsertFrame = {
    type: 'insert',
    char,
    index,
  };
  send(frame);
}

function App() {
  let [content, setContent] = useState<string>("");

  ws.onmessage = (event) => {
    const json = JSON.parse(event.data);
    const frame = json as ServerFrame;
    switch (frame.type) {
      case 'update':
        let frame = json as UpdateFrame;
        console.log(`new content: ${frame.content}`);
        setContent(frame.content);
        break;
    }
  }

  return (
    <div className="App">
      <textarea readOnly value={content} onKeyPress={(e) => insert(e.key, content.length)}/>
    </div>
  );
}

export default App;
