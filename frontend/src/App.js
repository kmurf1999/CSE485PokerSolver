import {Button, TextField, makeStyles, Box, StylesProvider, colors} from '@material-ui/core';
import List from '@material-ui/core/List';
import ListItem from '@material-ui/core/ListItem';
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
    case 4: return 'B';
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
  let rank = rankToChar(index % 13);
  let suit = suitToChar(Math.floor(index / 14));
    if (index === 99){
        rank = 1
        suit = 'B'
    }
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
  const { url, client_id } = await Join_game();
  return {client: W3CWebSocket(url), clientid: client_id};
}

const useStyles = makeStyles((theme) => ({
  root: {
    '& > *': {
      margin: theme.spacing(1),
    },
  },
  game: {
    width: '100%',
    display: 'grid',
    gridGap: '1em',
    gridTemplateCols: '80px 1fr',
    gridTemplateAreas: ' "history table" "history actions" '
  },
  table: {
    gridArea: 'table',
    height: 500,
    position: 'relative',
    display: 'grid',
    gridTemplateRows: '2fr 3fr 2fr',
    padding: '1em'
  },
  actions: {
    gridArea: 'actions'
  },
  history: {
    gridArea: 'history',
    height: '100%',
    maxHeight: '500px',
    overflow: 'auto'
  },
  player: {
    display: 'flex',
    justifyContent: 'center',
    alignItems: 'flex-end',
  },
  hero: {
    position: 'relative'
  },
  villan: {
    position: 'relative'
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
    background: 'white',
    borderRadius: '10px',
    padding: '2px 4px',
    marginRight: '4px',
    height: '25px',
    lineHeight: '25px',
    fontWeight: 'bold'
  },
  wager: {
    background: 'white',
    borderRadius: '10px',
    padding: '2px 4px',
    position: 'absolute',
    left: '-100%',
    height: '25px',
    lineHeight: '25px'
  },
  stack:{
    background: 'white',
    borderRadius: '10px',
    padding: '2px 4px',
    marginTop: '4px',
    marginBottom: '4px',
    height: '25px',
    lineHeight: '25px'
  }
}));

export default function App() {
   const classes = useStyles();
  let [client, setClient] = useState(null);
  let [clientId, setClientId] = useState(null);
  let [stacks, setStacks] = useState([0, 0]);
  let [wagers, setWagers] = useState([0, 0]);
  let [pot, setPot] = useState(0);
  let [boardCards, setBoardCards] = useState([99, 99, 99, 99, 99]);
  let [gameHistory, setGameHistory] = useState([]);
  let [heroCards, setHeroCards] = useState([99, 99]);
  let [villanCards, setVillanCards] = useState([99, 99]);
  let [pos, setPos] = useState(0);
  let [bet, setBet] = useState(0);

  useEffect(() => {
    if (client === null) {
      connect().then(({client: connectedClient, clientid}) => {
        connectedClient.onopen = () => {
          console.log('connected');
        }
        connectedClient.onclose = () => {
          console.log('disconnected');
        }
          setClient(connectedClient)
          setClientId(clientid)
      });
    } else {

        //CLIENT isn't null anymore
        client.onmessage = (message) => {
            HandleMessage(JSON.parse(message.data));

        }
        client.onopen = () => {
            console.log('connected');

        }

        client.onclose = () => {
            console.log('disconnected');
        }

        }


  });
  function HandleMessage(message) {
      //TODO handling messages now works, we have to implement what to do in each event
    const eventType = typeof message.event === "string" ? message.event : Object.keys(message.event)[0];

    switch (eventType) {
      case 'HandStart': {
        const { position, stacks: newStacks } = message.event['HandStart'];
        setPos(position.indexOf(clientId));
        setStacks(newStacks);
        setBoardCards([99, 99, 99, 99, 99]);
        setHeroCards([99, 99]);
        setVillanCards([99, 99]);
        setWagers([0, 0]);

        const actionString = 'Hand Starting';
        setGameHistory([...gameHistory, actionString]);
        break;
      }
      case 'HandEnd': {
        const { stacks: newStacks } = message.event['HandEnd'];
        setStacks(newStacks);
        setWagers([0, 0]);

        const actionString = 'Hand Over';
        setGameHistory([...gameHistory, actionString]);
        break;
      }
      case 'DealCards': {
        const { round, cards } = message.event['DealCards'];
        switch (round) {
            case 'PREFLOP':
                setHeroCards(cards)
                break;
            case 'FLOP':
                cards[3] = 99;
                cards[4] = 99;
                setWagers([0, 0]);
                setBoardCards(cards);
                break;
            case 'TURN':
                cards[4] = 99;
                setWagers([0, 0]);
                setBoardCards(cards);
                break;
            case 'RIVER':
                setWagers([0, 0]);
                setBoardCards(cards);
                break;
            default: break;
        }
        break;
      }
      case 'PostBlinds': {
        const { pot: newPot, stacks: newStacks, wagers: newWagers } = message.event['PostBlinds'];
        setStacks(newStacks);
        setWagers(newWagers);
        setPot(newPot);
        break;
      }
      case 'RequestAction':
      case 'SendAction':
        break;
      case 'AlertAction': {
        const { action, pot: newPot, stacks: newStacks, wagers: newWagers } = message.event['AlertAction'];

        const player = message.from === clientId ? 'Hero' : 'Villian';
        const actionString = `${player} ${String(action)}`;

        setGameHistory([...gameHistory, actionString]);
        setStacks(newStacks);
        setWagers(newWagers);
        setPot(newPot);
        break;
      }
   
      default: break;
    }
  }
  function HandleChange(e){
      setBet(e.target.value);
  }
  function BET(amount) {
    return "BET"
  }
  function handleClick(){
    console.log(bet)
    client.send(JSON.stringify({ SendAction: { action: { BET: parseInt(bet) }}}))
  }
  function Fold() {
    if (client !== null) {
        client.send(JSON.stringify({SendAction: {action: 'FOLD' }}))
    }
  }
  function Check() {
    if (client !== null) {
        client.send(JSON.stringify({SendAction: {action: 'CHECK' }}))
    }
  }
  function Call() {
    if (client !== null) {
        client.send(JSON.stringify({SendAction: {action: 'CALL'}}))
    }
  }

  return (
    <div className={classes.game}>
      <div className={classes.table}>
        <div className={classes.felt}/>
        <div className={classes.player} style={{gridRow: 1}}>
          <div className={classes.villan}>
            <div className={classes.stack}>
              Villan: <strong>${stacks[1 - pos]}</strong>
            </div>
            <div className={classes.privateCards}>
              <Card index={villanCards[0]}/>
              <Card index={villanCards[1]}/>
            </div>
            <div className={classes.wager} style={{bottom: 0}}>
              Bet: <strong>${wagers[1 - pos]}</strong>
            </div>
          </div>
        </div>
        <div className={classes.board}>
            <div className={classes.pot}>
              ${pot}
            </div>
            <div className={classes.boardCards}>
              <Card index={boardCards[0]}/>
              <Card index={boardCards[1]}/>
              <Card index={boardCards[2]}/>
              <Card index={boardCards[3]}/>
              <Card index={boardCards[4]}/>
            </div>
        </div>
        <div className={classes.player} style={{gridRow: 3}}>
          <div className={classes.hero}>
            <div className={classes.privateCards}>
              <Card index={heroCards[0]}/>
              <Card index={heroCards[1]}/>
            </div>
            <div className={classes.stack}>
              Hero: <strong>${stacks[pos]}</strong>
            </div>
            <div className={classes.wager} style={{top: 0}}>
              Bet: <strong>${wagers[pos]}</strong>
            </div>
          </div>
        </div>
      </div>


    <div className={classes.actions}>
      <Box p = {1} display="flex" alignItems="center" justifyContent="center">
        <Button onClick={Fold} variant = "outlined" color="primary">Fold</Button>
        <Button onClick={Check} variant = "outlined" color="primary">Check</Button>
        <Button onClick={Call} variant = "outlined" color="primary">Call</Button>
        <Button onClick={handleClick} variant = "outlined" color="primary">Bet</Button>
      </Box>

      <Box p = {1} display="flex" alignItems="center" justifyContent="center">
        <TextField onChange={HandleChange} id="Bet-Entry" label = "Bet Amount" variant = "outlined" />
      </Box>

    </div>

    <List className={classes.history}>
        {gameHistory.map((action, index) => 
          <ListItem key={index}>
            {action}
          </ListItem>
        )}
    </List>


    </div>
  );
}
