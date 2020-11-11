import { Button, TextField, makeStyles, Box } from '@material-ui/core';
import { Ellipse } from 'react-shapes';
import './App.css';
import { ReactComponent as card } from './2C.svg'

const useStyles = makeStyles((theme) => ({
  root: {
    '& > *': {
      margin: theme.spacing(1),

    },
  },
}));

export default function App() {
  const classes = useStyles();

  return (
    <div>
      <card className = 'test-card' ariaLabel='card' />
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
