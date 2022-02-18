import React, { useState } from 'react';
import logo from './logo.svg';
import './App.css';

type ClientFrameType = 'insert' | 'open';

interface ClientFrame {
  type: ClientFrameType
};

interface InsertFrame extends ClientFrame {
  cursor: number,
  content: string,
}

interface Open extends ClientFrame {
  url: string,
}

const ws = new WebSocket("ws://localhost:8080")

const insert = (buffer: string) => {
  let frame: InsertFrame = {
    type: 'insert',
    content: buffer,
    cursor: 0
  };
  const json = JSON.stringify(frame);
  ws.send(json)

}

function App() {
  let [content, setContent] = useState('');

  return (
    <div className="App">
      <input type="text" value={content} onChange={(e) => setContent(e.target.value)}/>
      <button onClick={(_) => insert(content)}>Insert</button>
    </div>
  );
}

export default App;
