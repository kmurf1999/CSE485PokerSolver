import {Button, TextField, makeStyles, Box, StylesProvider, colors} from '@material-ui/core';
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
    padding: '1em',
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
      position: 'relative',
      gridRow: 4

  },
    stack:{
        position: 'relative',
        gridRow: 5

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
  let [ourCards, setOurCards] = useState([99, 99]);
  let [round, setRound ] = useState(null);
  let [pos_index, setPos] = useState();
  let [bet, setBet] = useState(0);

  useEffect(() => {
    if (client === null) {
      connect().then(({client, clientid}) => {


        client.onopen = () => {
          console.log('connected');

        }
        client.onclose = () => {
          console.log('disconnected');
        }

          setClient(client)
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
      case 'GameStart':
        break;
      case 'GameEnd':
        break;
      case 'HandStart':
          let { position } = message.event['HandStart'];
          let pos_index;
          if (pos_index = position.indexOf(clientId)){
              setPos(pos_index)
          } else {
              setPos(pos_index)
          }
        break;
      case 'HandEnd':
        break;
      case 'DealCards': {
        const { round, cards } = message.event['DealCards'];
          setRound(round)
        switch (round) {
            case 'PREFLOP':
                setOurCards(cards)
                break;
            case 'FLOP':
                cards[3] = 99
                cards[4] = 99
                setBoardCards(cards)
                break;
            case 'TURN':
                cards[4] = 99
                setBoardCards(cards)
                break;
            case 'RIVER':
                setBoardCards(cards)

                //Known issue: Cards disappear before players actions in RIVER round
                cards[0] = 99
                cards[1] = 99
                cards[2] = 99
                cards[3] = 99
                cards[4] = 99
                break;
        }
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
            //TODO: Implement payloads for (RAISE) Button
            window.alert("Your turn")
      case 'SendAction':
        break;
      case 'AlertAction': {
        const { action, pot, stacks, wagers } = message.event['AlertAction'];
        setStacks(stacks);
        setWagers(wagers);
        console.log(wagers)
        setPot(pot);
        break;
      }
   
      default: break;
    }
  }

  function HandleChange(e){
      setBet(e.target.value);
  }


  //BET
    function BET(amount) {

      return "BET"
    }

  function handleClick(){
      console.log(bet)
          //client.send(JSON.stringify(JSON.parse( "{\"SendAction\": {\"action\": \"BET\"}}")))
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
        <div className={classes.villan}>
          <div className={classes.privateCards}>
            <Card index={99}/>
            <Card index={99}/>
          </div>
          <div className={classes.wager}>
          </div>
        </div>
        <div className={classes.board}>
            <div className={classes.pot}>
              Pot: {pot}
            </div>
            <div className={classes.boardCards}>
              <Card index={boardCards[0]}/>
              <Card index={boardCards[1]}/>
              <Card index={boardCards[2]}/>
              <Card index={boardCards[3]}/>
              <Card index={boardCards[4]}/>
            </div>
        </div>
        <div className={classes.hero}>
            <div className={classes.privateCards}>
              <Card index={ourCards[0]}/>
              <Card index={ourCards[1]}/>
            </div>
        </div>
      </div>


      <div className={classes.actions}>
        <Box p = {1} display="flex" alignItems="center" justifyContent="center">
        <Button onClick={Fold} variant = "outlined" color="primary">Fold</Button>
        <Button onClick={Check} variant = "outlined" color="primary">Check</Button>
        <Button onClick={Call} variant = "outlined" color="primary">Call</Button>
            <div className={classes.stack}>
                Stack: {stacks[pos_index]}
            </div>
            <div className={classes.wager}>
                Wager: {wagers[pos_index]}
            </div>
        </Box>

        <Box p = {1} display="flex" alignItems="center" justifyContent="center">
          <Button onClick={BET} variant = "outlined" color="primary">Min Bet</Button>
          <Button onClick={BET} variant = "outlined" color="primary">Bet Half Pot</Button>
          <Button onClick={BET} variant = "outlined" color="primary">Bet Pot</Button>
          <Button onClick={BET} variant = "outlined" color="primary">All In</Button>
          <TextField onChange={HandleChange} id="Bet-Entry" label = "Bet Amount" variant = "outlined" />
        </Box>

        <Box p = {1} display = "flex" alignItems="center" justifyContent="center">
            <Button onClick={handleClick} variant = "outlined" color="primary">Bet</Button>
        </Box>


      </div>

      <div className={classes.history}>

      </div>


      </div>
  );
}
