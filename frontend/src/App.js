import { Button, TextField, makeStyles, Box } from '@material-ui/core';
import { Ellipse } from 'react-shapes';
import './App.css';
import { w3cwebsocket as W3CWebSocket } from "websocket";
import React, { useEffect, useState } from 'react';
// import { ReactComponent as card } from './2C.svg'

const BASE_URI = 'http://localhost:3001';

function create_game() {
  return new Promise((resolve, reject) => {
    fetch(`${BASE_URI}/create`, {
      method: 'POST',
    })
    .then(res => res.json())
    .then(res => resolve(res))
    .catch(err => reject(err));
  })
}

function join_game(game_id) {
  return new Promise((resolve, reject) => {
    fetch(`${BASE_URI}/join`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({ game_id })
    })
    .then(res => res.json())
    .then(res => resolve(res))
    .catch(err => reject(err));
  })
}

async function connect() {
  const { game_id } = await create_game();
  console.log(game_id);
  let { url, client_id } = await join_game(game_id);
  return new W3CWebSocket(url);
}

const useStyles = makeStyles((theme) => ({
  root: {
    '& > *': {
      margin: theme.spacing(1),
    },
  },
}));

export default function App() {
  // const classes = useStyles();
  let [client, setClient] = useState(null);
  let [stacks, setStacks] = useState([0, 0]);
  let [wagers, setWagers] = useState([0, 0]);
  let [pot, setPot] = useState(0);
  let [cards, setCards] = useState([52, 52, 52, 52, 52, 52, 52]);

  useEffect(() => {
    if (client === null) {
      connect().then(client => {
        client.onmessage = (message) => handleMessage(JSON.parse(message.data));
        client.onopen = () => {
          console.log('connected');
        }
        client.onclose = () => {
          console.log('disconnected');
        }
        setClient(client);
      });
    }
  });

  function handleMessage(message) {
    const eventType = typeof message.event === "string" ? message.event : Object.keys(message.event)[0];
    switch (eventType) {
      case 'GameStart':
        break;
      case 'GameEnd':
        break;
      case 'HandStart':
        break;
      case 'HandEnd':
        break;
      case 'DealCards': {
        const { cards, round } = message.event['DealCards'];
        setCards(cards);
        break;
      }
      case 'PostBlinds': {
        const { blinds, pot, stacks, wagers } = message.event['PostBlinds'];
        setStacks(stacks);
        setWagers(wagers);
        setPot(pot);
        break;
      }
      case 'RequestAction':
        break;
      case 'SendAction':
        break;
      case 'AlertAction': {
        const { action, pot, stacks, wagers } = message.event['AlertAction'];
        setStacks(stacks);
        setWagers(wagers);
        setPot(pot);
        break;
      }
   
      default: break;
    }
  }

  return (
    <div>
      {/* <card className = 'test-card' /> */}
      <Box p = {1} display="flex" alignItems = "center" justifyContent = "center">
        
        <Ellipse rx={350} ry={200} fill={{color:'#14B32D'}} stroke={{color:'#14B32D'}} strokeWidth={3} />
      </Box>

      <Box m = {4} display="flex" alignItems = "Left" justifyContent = "Left">
        Outcome:
      </Box>

      <Box p = {1} display="flex" alignItems="center" justifyContent="center">
      <Button variant = "outlined" color="primary">Next Hand</Button>
      <Button variant = "outlined" color="primary">Fold</Button>
      <Button variant = "outlined" color="primary">Check</Button>
      <Button variant = "outlined" color="primary">Call</Button>
      <Button variant = "outlined" color="primary">Peek</Button>
      </Box>

      <Box p = {1} display="flex" alignItems="center" justifyContent="center">
      <Button variant = "outlined" color="primary">Min Bet</Button>
      <Button variant = "outlined" color="primary">Bet Half Pot</Button>
      <Button variant = "outlined" color="primary">Bet Pot</Button>
      <Button variant = "outlined" color="primary">All In</Button>
      <Button variant = "outlined" color="primary">Bet</Button>
      </Box>

      <Box p = {1} display = "flex" alignItems="center" justifyContent="center">
      <TextField id="Bet-Entry" label = "Bet Amount" variant = "outlined" />
      </Box>
      </div>
  );
}
