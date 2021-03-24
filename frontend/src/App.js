import {Button, Paper, InputBase, Typography, TextField, makeStyles, Box, Divider, StylesProvider, colors, FormControl, ButtonGroup} from '@material-ui/core';
import OutlinedInput from '@material-ui/core/OutlinedInput';
import InputLabel from '@material-ui/core/InputLabel';
import InputAdornment from '@material-ui/core/InputAdornment';
import List from '@material-ui/core/List';
import ListItem from '@material-ui/core/ListItem';
import { Ellipse } from 'react-shapes';
import Slider from '@material-ui/core/Slider';
import './App.css';
import { w3cwebsocket as W3CWebSocket } from "websocket";
import React, { useEffect, useState } from 'react';

const BASE_URI = 'http://localhost:3001';

function suitToChar(suit) {
  switch(suit) {
    case 0: return 'S';
    case 1: return 'H';
    case 2: return 'D';
    case 3: return 'C';
    case 4: return 'B';
    default: return '';
  }
}

function rankToChar(rank) {
  switch(parseInt(rank)) {
    case 13: return '2';
    case 12: return 'A';
    case 11: return 'K'; //+2 ==> A
    case 10: return 'Q';
    case 9: return 'J';
    case 8: return 'T';
    default: return String(parseInt(rank + 2));
  }
}

function Card({index}) {
  //
  const classes = useStyles();
  let rank = rankToChar(index / 4);
  let suit = suitToChar(Math.floor(index % 4));
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
    position: 'relative',
    width: '100%',
    maxWidth: 1200,
    margin: '0 auto',
    display: 'grid',
    gridGap: '1em',
    gridTemplateColumns: '1em 300px 1fr 1em',
    gridTemplateAreas: ' ". history table ." ". history actions ." ',
    paddingTop: '1em'
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
  actionTimer: {
    maxWidth: 500,
    margin: '0 auto'
  },
  actionInputs: {
    display: 'flex',
    flexDirection: 'row',
    justifyContent: 'space-around',
    maxWidth: 500,
    margin: '0 auto'
  },
  betEntryPaper: {
    padding: '2px 4px',
    display: 'flex',
    alignItems: 'center',
    width: 200,
  },
  betEntryInput: {
    flex: 1,
    marginLeft: '4px'
  },
  betEntryDivider: {
    height: 28,
    margin: 4,
  },
  betEntryButton: {
    
  },
  history: {
    gridArea: 'history',
    height: '100%',
    maxHeight: '100vh',
    overflow: 'auto',
    padding: '8px'
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
    gridRow: 2,
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
  let [amount, setAmount] = useState(0);

  let [timer, setTimer] = useState(null);
  let [timeLeft, setTimeLeft] = useState(30);

  const timerTick = () => {
    if (timeLeft === 0) {
      setTimer(clearInterval(timer));
    } else {
      setTimeLeft(timeLeft => timeLeft - 1);
    }
  }

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
        const { round: round, cards: cards, test: test } = message.event['DealCards'];
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
      case 'RequestAction': {
        setTimer(setInterval(timerTick, 1000));
        break;
      }
      case 'SendAction':
        break;
      case 'AlertAction': {
        setTimer(clearInterval(timer));
        setTimeLeft(30);
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
  function BetOrRaise(){
    if (raiseValid) {
      client.send(JSON.stringify({ SendAction: { action: { RAISE: parseInt(amount) }}}));
    } else {
      client.send(JSON.stringify({ SendAction: { action: { BET: parseInt(amount) }}}));
    }
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

  const raiseValid = wagers[1 - pos] > wagers[pos];
  const betValid = wagers[1 - pos] === 0;
  const checkValid = wagers[1 - pos] === 0;
  const callValid = wagers[1 - pos] > wagers[pos];
  const foldValid = wagers[1 - pos] > wagers[pos];

  const BetOrRaiseText = raiseValid ? 'Raise' : 'Bet';

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
      <Box className={classes.actionTimer}>
        <Slider value={timeLeft * (100 / 30)} onChange={null}/>
      </Box>

      <div className={classes.actionInputs}>
        <Button disabled={!foldValid} variant="contained" onClick={Fold}>Fold</Button>
        <Button disabled={!checkValid} variant="contained" onClick={Check}>Check</Button>
        <Button disabled={!callValid} variant="contained" onClick={Call}>Call</Button>
        <Paper component="form" className={classes.betEntryPaper}>
          <InputBase
            placeholder="Bet size"
            inputProps={{ 'aria-label': 'bet size' }}
            className={classes.betEntryInput}
            type="num"
            value={amount}
            onChange={e => setAmount(e.target.value)}
            startAdornment={<InputAdornment position="start">$</InputAdornment>}
          />
          <Divider className={classes.betEntryDivider} orientation="vertical" />
          <Button className={classes.betEntryButton} variant="contained" disableElevation color="primary" disabled={!betValid && !raiseValid} onClick={BetOrRaise}>{BetOrRaiseText}</Button>
        </Paper>
      </div>

    </div>

    <Paper className={classes.history}>
      <Typography>
        Game History
      </Typography>
      <List>
          {gameHistory.map((action, index) => 
            <ListItem key={index}>
              {action}
            </ListItem>
          )}
      </List>
    </Paper>



    </div>
  );
}
