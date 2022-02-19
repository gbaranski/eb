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

type ClientFrameType = 'set' | 'insert' | 'open';

interface ClientFrame {
  type: ClientFrameType
};

interface SetFrame extends ClientFrame {
  content: string,
}

interface InsertFrame extends ClientFrame {
  cursor: number,
  content: string,
}

interface Open extends ClientFrame {
  url: string,
}

const ws = new WebSocket("ws://localhost:8080")

const set = (content: string) => {
  let frame: SetFrame = {
    type: 'set',
    content,
  };
  const json = JSON.stringify(frame);
  ws.send(json)
}

function App() {
  let [content, setContent] = useState('');

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
      <input type="text" id="code" value={content} onChange={(e) => setContent(e.target.value)}/>
      <button onClick={(_) => set(content)}>Save</button>
    </div>
  );
}

export default App;
