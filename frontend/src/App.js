import { Button, TextField, makeStyles, Box, StylesProvider } from '@material-ui/core';
import { Ellipse } from 'react-shapes';
import './App.css';
import { w3cwebsocket as W3CWebSocket } from "websocket";
import React, { useEffect, useState } from 'react';

const BASE_URI = 'http://localhost:3001';

function suitToChar(suit) {
  switch(suit) {
    case 0: return 'S';
    case 1: return 'H';
    case 2: return 'C';
    case 3: return 'D';
    default: return '';
  }
}

function rankToChar(rank) {
  switch(rank) {
    case 12: return 'A';
    case 11: return 'K';
    case 10: return 'Q';
    case 9: return 'J';
    case 8: return 'T';
    default: return String(rank + 2);
  }
}


function Card({ index }) {
  const classes = useStyles();
  const rank = rankToChar(index % 13);
  const suit = suitToChar(Math.floor(index / 14));
  return (
    <div className={classes.card}>
      <img 
        style={{objectFit: 'contain', height: 100, width: 'auto'}}
        alt={`${rank}${suit}`}
        src={`cards/${rank}${suit}.svg`}
        />
    </div>
  );
}

/*function create_game() {
  return new Promise((resolve, reject) => {
    fetch(`${BASE_URI}/create`, {
      method: 'POST',
    })
    .then(res => res.json())
    .then(res => resolve(res))
    .catch(err => reject(err));
  })
}*/

function Join_game(game_id) {
    return new Promise((resolve, reject) => {
        fetch('http://localhost:3001/join', {
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
  //const { game_id } = await Join_game();
  //console.log(game_id);
  let { url } = await Join_game();
  return {client: W3CWebSocket(url), clientId: url};
}

const useStyles = makeStyles((theme) => ({
  root: {
    '& > *': {
      margin: theme.spacing(1),
    },
  },
  game: {
    padding: '1em',
    display: 'grid',
    gridGap: '1em',
    gridTemplateCols: '80px 1fr',
    gridTemplateAreas: ' "history table" "history actions" '
  },
  table: {
    gridArea: 'table',
    width: '100%',
    height: 500,
    position: 'relative',
    display: 'grid',
    gridTemplateRows: '2fr 3fr 2fr',
  },
  actions: {
    gridArea: 'actions'
  },
  history: {
    gridArea: 'history',
    height: '100%',
    background: 'grey'
  },
  hero: {
    position: 'relative',
    display: 'flex',
    justifyContent: 'center',
    alignItems: 'flex-end',
    gridRow: 3
  },
  villan: {
    alignItems: 'flex-start',
    position: 'relative',
    display: 'flex',
    justifyContent: 'center',
    gridRow: 1
  },
  board: {
    position: 'relative',
    display: 'flex',
    justifyContent: 'center',
    alignItems: 'center',
    gridRow: 2
  },
  felt: {
    position: 'absolute',
    width: '100%',
    height: '100%',
    background: 'green',
    borderRadius: '50%',
    zIndex: -1
  },
  privateCards: {
    display: 'flex',
    flexDirection: 'row',
  },
  boardCards: {
    display: 'flex',
    flexDirection: 'row',
    alignItems: 'center',
  },
  card: {
    maxHeight: 100,
  },
  pot: {

  },
  wager: {

  }
}));

export default function App() {
  // const classes = useStyles();
  let [client, setClient] = useState(null);
  let [clientId, setClientId] = useState(null);
  let [stacks, setStacks] = useState([0, 0]);
  let [wagers, setWagers] = useState([0, 0]);
  let [pot, setPot] = useState(0);
  let [boardCards, setBoardCards] = useState([52, 52, 52, 52, 52]);
  let [ourCards, setOurCards] = useState([52, 52]);

  const classes = useStyles();

  useEffect(() => {
    if (client === null) {
      connect().then(({ client, clientId}) => {
        client.onmessage = (message) => handleMessage(JSON.parse(message.data));
        client.onopen = () => {
          console.log('connected');
        }
        client.onclose = () => {
          console.log('disconnected');
        }
        setClient(client);
        setClientId(clientId);
      });
    }
  });

  function handleMessage(message) {
      //TODO handling messages now works, we have to implement what to do in each event
    const eventType = typeof message.event === "string" ? message.event : Object.keys(message.event)[0];
    switch (eventType) {
      case 'GameStart':
          console.log("FINALLLYYYYY")
        break;
      case 'GameEnd':
        break;
      case 'HandStart':
        break;
      case 'HandEnd':
        break;
      case 'DealCards': {
        const { cards, round } = message.event['DealCards'];
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
    <div className={classes.game}>
      <div className={classes.table}>
        <div className={classes.felt}/>
        <div className={classes.villan}>
          <div className={classes.privateCards}>
            <Card index={0}/>
            <Card index={1}/>
          </div>
          <div className={classes.wager}>
          </div>
        </div>
        <div className={classes.board}>
            <div className={classes.pot}>
              Pot: {pot}
            </div>
            <div className={classes.boardCards}>
              <Card index={0}/>
              <Card index={1}/>
              <Card index={0}/>
              <Card index={1}/>
              <Card index={0}/>
            </div>
        </div>
        <div className={classes.hero}>
            <div className={classes.wager}>
              
            </div>
            <div className={classes.privateCards}>  
              <Card index={0}/>
              <Card index={1}/>
            </div>
        </div>
      </div>


      <div className={classes.actions}>
        <Box p = {1} display="flex" alignItems="center" justifyContent="center">
        <Button variant = "outlined" color="primary">Fold</Button>
        <Button variant = "outlined" color="primary">Check</Button>
        <Button variant = "outlined" color="primary">Call</Button>
        </Box>

        <Box p = {1} display="flex" alignItems="center" justifyContent="center">
          <Button variant = "outlined" color="primary">Min Bet</Button>
          <Button variant = "outlined" color="primary">Bet Half Pot</Button>
          <Button variant = "outlined" color="primary">Bet Pot</Button>
          <Button variant = "outlined" color="primary">All In</Button>
          <TextField id="Bet-Entry" label = "Bet Amount" variant = "outlined" />
        </Box>

        <Box p = {1} display = "flex" alignItems="center" justifyContent="center">
          <Button variant = "outlined" color="primary">Bet</Button>
        </Box>
      </div>

      <div className={classes.history}>

      </div>


      </div>
  );
}
