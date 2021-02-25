import { Button, TextField, Box } from '@material-ui/core';
import './App.css';
import table from './cards/Table.svg';
import card from './cards/2B.svg';
import card1 from './cards/2C.svg';
import card2 from './cards/2D.svg';
import card3 from './cards/2H.svg';
import card4 from './cards/2S.svg';
import card5 from './cards/3C.svg';
import card6 from './cards/3D.svg';
import card7 from './cards/3H.svg';
import card8 from './cards/3S.svg';
import card9 from './cards/4C.svg';
import card10 from './cards/4D.svg';
import card11 from './cards/4H.svg';
import card12 from './cards/4S.svg';
import card13 from './cards/5C.svg';
import card14 from './cards/5D.svg';
import card15 from './cards/5H.svg';
import card16 from './cards/5S.svg';
import card17 from './cards/6C.svg';
import card18 from './cards/6D.svg';
import card19 from './cards/6H.svg';
import card20 from './cards/6S.svg';
import card21 from './cards/7C.svg';
import card22 from './cards/7D.svg';
import card23 from './cards/7H.svg';
import card24 from './cards/7S.svg';
import card25 from './cards/8C.svg';
import card26 from './cards/8D.svg';
import card27 from './cards/8H.svg';
import card28 from './cards/8S.svg';
import card29 from './cards/9C.svg';
import card30 from './cards/9D.svg';
import card31 from './cards/9H.svg';
import card32 from './cards/9S.svg';
import card33 from './cards/TC.svg';
import card34 from './cards/TD.svg';
import card35 from './cards/TH.svg';
import card36 from './cards/TS.svg';
import card37 from './cards/JC.svg';
import card38 from './cards/JD.svg';
import card39 from './cards/JH.svg';
import card40 from './cards/JS.svg';
import card41 from './cards/QC.svg';
import card42 from './cards/QD.svg';
import card43 from './cards/QH.svg';
import card44 from './cards/QS.svg';
import card45 from './cards/KC.svg';
import card46 from './cards/KD.svg';
import card47 from './cards/KH.svg';
import card48 from './cards/KS.svg';
import card49 from './cards/AC.svg';
import card50 from './cards/AD.svg';
import card51 from './cards/AH.svg';
import card52 from './cards/AS.svg';

var hand = "ACAH"; //Cards in the players hand
var board = "KHAD9C5C6C"; //Cards on the table
var botHand = "KHKD"; //Bot's card
var pot; //Chips bet
var stack; //Chips player has
var resultText = "Sample text";

var playerCard1, playerCard2;
var botCard1, botCard2;
var boardCard1, boardCard2, boardCard3, boardCard4, boardCard5;

function nextHand() {
  console.log('Continue to next hand');
  //window.location.reload();
}

function fold() {
  console.log('Hand Folded');
}

function check() {
  console.log('Check');
}

function call() {
  console.log('Calling bet');
}

function peek() {
  console.log('Peeking');
}

function minBet() {
  console.log('Betting minimum');
  //Pass back bet size
  //Recieve back updated stack and pot size
}

function halfPot(){
  console.log('Betting half the pot');
  //Pass back bet size
  //Recieve back updated stack and pot size
}

function fullPot() {
  console.log('Betting the pot');
  //Pass back bet sizes
  //Recieve back updated stack and pot size
}

function allIn() {
  console.log('Going all in');
  //Pass back bet size
  //Recieve back updated stack and pot size
}

function specificBet() {
  var bet = document.getElementById("Bet-Entry").value;
  console.log('Betting: ' + bet);
  //Pass back bet size
  //Recieve back updated stack and pot size
}

function cardDisplayer(set, string) {
  if(set === 1) { //Updates players cards
    if (string.substring(0,2) === "2C")
      playerCard1 = card1;
    else if (string.substring(0,2) === "2D")
      playerCard1 = card2;
    else if (string.substring(0,2) === "2H")
        playerCard1 = card3;
    else if (string.substring(0,2) === "2S")
        playerCard1 = card4;
    else if (string.substring(0,2) === "3C")
        playerCard1 = card5;
    else if (string.substring(0,2) === "3D")
        playerCard1 = card6;
    else if (string.substring(0,2) === "3H")
        playerCard1 = card7;
    else if (string.substring(0,2) === "3S")
        playerCard1 = card8;
    else if (string.substring(0,2) === "4C")
        playerCard1 = card9;
    else if (string.substring(0,2) === "4D")
        playerCard1 = card10;
    else if (string.substring(0,2) === "4H")
        playerCard1 = card11;
    else if (string.substring(0,2) === "4S")
        playerCard1 = card12;
    else if (string.substring(0,2) === "5C")
        playerCard1 = card13;
    else if (string.substring(0,2) === "5D")
        playerCard1 = card14;
    else if (string.substring(0,2) === "5H")
        playerCard1 = card15;
    else if (string.substring(0,2) === "5S")
        playerCard1 = card16;
    else if (string.substring(0,2) === "6C")
        playerCard1 = card17;
    else if (string.substring(0,2) === "6D")
        playerCard1 = card18;
    else if (string.substring(0,2) === "6H")
        playerCard1 = card19;
    else if (string.substring(0,2) === "6S")
        playerCard1 = card20;
    else if (string.substring(0,2) === "7C")
        playerCard1 = card21;
    else if (string.substring(0,2) === "7D")
        playerCard1 = card22;
    else if (string.substring(0,2) === "7H")
        playerCard1 = card23;
    else if (string.substring(0,2) === "7S")
        playerCard1 = card24;
    else if (string.substring(0,2) === "8C")
        playerCard1 = card25;
    else if (string.substring(0,2) === "8D")
        playerCard1 = card26;
    else if (string.substring(0,2) === "8H")
        playerCard1 = card27;
    else if (string.substring(0,2) === "8S")
        playerCard1 = card28;
    else if (string.substring(0,2) === "9C")
        playerCard1 = card29;
    else if (string.substring(0,2) === "9D")
        playerCard1 = card30;
    else if (string.substring(0,2) === "9H")
        playerCard1 = card31;
    else if (string.substring(0,2) === "9S")
        playerCard1 = card32;
    else if (string.substring(0,2) === "TC")
        playerCard1 = card33;
    else if (string.substring(0,2) === "TD")
        playerCard1 = card34;
    else if (string.substring(0,2) === "TH")
        playerCard1 = card35;
    else if (string.substring(0,2) === "TS")
        playerCard1 = card36;
    else if (string.substring(0,2) === "JC")
        playerCard1 = card37;
    else if (string.substring(0,2) === "JD")
        playerCard1 = card38;
    else if (string.substring(0,2) === "JH")
        playerCard1 = card39;
    else if (string.substring(0,2) === "JS")
        playerCard1 = card40;
    else if (string.substring(0,2) === "QC")
        playerCard1 = card41;
    else if (string.substring(0,2) === "QD")
        playerCard1 = card42;
    else if (string.substring(0,2) === "QH")
        playerCard1 = card43;
    else if (string.substring(0,2) === "QS")
        playerCard1 = card44;
    else if (string.substring(0,2) === "KC")
        playerCard1 = card45;
    else if (string.substring(0,2) === "KD")
        playerCard1 = card46;
    else if (string.substring(0,2) === "KH")
        playerCard1 = card47;
    else if (string.substring(0,2) === "KS")
        playerCard1 = card48;
    else if (string.substring(0,2) === "AC")
        playerCard1 = card49;
    else if (string.substring(0,2) === "AD")
        playerCard1 = card50;
    else if (string.substring(0,2) === "AH")
        playerCard1 = card51;
    else if (string.substring(0,2) === "AS")
        playerCard1 = card52;

        if (string.substring(2,4) === "2C")
          playerCard2 = card1;
        else if (string.substring(2,4) === "2D")
          playerCard2 = card2;
        else if (string.substring(2,4) === "2H")
            playerCard2 = card3;
        else if (string.substring(2,4) === "2S")
            playerCard2 = card4;
        else if (string.substring(2,4) === "3C")
            playerCard2 = card5;
        else if (string.substring(2,4) === "3D")
            playerCard2 = card6;
        else if (string.substring(2,4) === "3H")
            playerCard2 = card7;
        else if (string.substring(2,4) === "3S")
            playerCard2 = card8;
        else if (string.substring(2,4) === "4C")
            playerCard2 = card9;
        else if (string.substring(2,4) === "4D")
            playerCard2 = card10;
        else if (string.substring(2,4) === "4H")
            playerCard2 = card11;
        else if (string.substring(2,4) === "4S")
            playerCard2 = card12;
        else if (string.substring(2,4) === "5C")
            playerCard2 = card13;
        else if (string.substring(2,4) === "5D")
            playerCard2 = card14;
        else if (string.substring(2,4) === "5H")
            playerCard2 = card15;
        else if (string.substring(2,4) === "5S")
            playerCard2 = card16;
        else if (string.substring(2,4) === "6C")
            playerCard2 = card17;
        else if (string.substring(2,4) === "6D")
            playerCard2 = card18;
        else if (string.substring(2,4) === "6H")
            playerCard2 = card19;
        else if (string.substring(2,4) === "6S")
            playerCard2 = card20;
        else if (string.substring(2,4) === "7C")
            playerCard2 = card21;
        else if (string.substring(2,4) === "7D")
            playerCard2 = card22;
        else if (string.substring(2,4) === "7H")
            playerCard2 = card23;
        else if (string.substring(2,4) === "7S")
            playerCard2 = card24;
        else if (string.substring(2,4) === "8C")
            playerCard2 = card25;
        else if (string.substring(2,4) === "8D")
            playerCard2 = card26;
        else if (string.substring(2,4) === "8H")
            playerCard2 = card27;
        else if (string.substring(2,4) === "8S")
            playerCard2 = card28;
        else if (string.substring(2,4) === "9C")
            playerCard2 = card29;
        else if (string.substring(2,4) === "9D")
            playerCard2 = card30;
        else if (string.substring(2,4) === "9H")
            playerCard2 = card31;
        else if (string.substring(2,4) === "9S")
            playerCard2 = card32;
        else if (string.substring(2,4) === "TC")
            playerCard2 = card33;
        else if (string.substring(2,4) === "TD")
            playerCard2 = card34;
        else if (string.substring(2,4) === "TH")
            playerCard2 = card35;
        else if (string.substring(2,4) === "TS")
            playerCard2 = card36;
        else if (string.substring(2,4) === "JC")
            playerCard2 = card37;
        else if (string.substring(2,4) === "JD")
            playerCard2 = card38;
        else if (string.substring(2,4) === "JH")
            playerCard2 = card39;
        else if (string.substring(2,4) === "JS")
            playerCard2 = card40;
        else if (string.substring(2,4) === "QC")
            playerCard2 = card41;
        else if (string.substring(2,4) === "QD")
            playerCard2 = card42;
        else if (string.substring(2,4) === "QH")
            playerCard2 = card43;
        else if (string.substring(2,4) === "QS")
            playerCard2 = card44;
        else if (string.substring(2,4) === "KC")
            playerCard2 = card45;
        else if (string.substring(2,4) === "KD")
            playerCard2 = card46;
        else if (string.substring(2,4) === "KH")
            playerCard2 = card47;
        else if (string.substring(2,4) === "KS")
            playerCard2 = card48;
        else if (string.substring(2,4) === "AC")
            playerCard2 = card49;
        else if (string.substring(2,4) === "AD")
            playerCard2 = card50;
        else if (string.substring(2,4) === "AH")
            playerCard2 = card51;
        else if (string.substring(2,4) === "AS")
            playerCard2 = card52;

  } else if (set ===2) { //Updates Board cards
    if(string === ""){
      boardCard1 = card;
      boardCard2 = card;
      boardCard3 = card;
      boardCard4 = card;
      boardCard5 = card;
    } else {
      if(string.length >= 6) {

      if (string.substring(0,2) === "2C")
        botCard1 = card1;
      else if (string.substring(0,2) === "2D")
        boardCard1 = card2;
      else if (string.substring(0,2) === "2H")
          boardCard1 = card3;
      else if (string.substring(0,2) === "2S")
          boardCard1 = card4;
      else if (string.substring(0,2) === "3C")
          boardCard1 = card5;
      else if (string.substring(0,2) === "3D")
          boardCard1 = card6;
      else if (string.substring(0,2) === "3H")
          boardCard1 = card7;
      else if (string.substring(0,2) === "3S")
          boardCard1 = card8;
      else if (string.substring(0,2) === "4C")
          boardCard1 = card9;
      else if (string.substring(0,2) === "4D")
          boardCard1 = card10;
      else if (string.substring(0,2) === "4H")
          boardCard1 = card11;
      else if (string.substring(0,2) === "4S")
          boardCard1 = card12;
      else if (string.substring(0,2) === "5C")
          boardCard1 = card13;
      else if (string.substring(0,2) === "5D")
          boardCard1 = card14;
      else if (string.substring(0,2) === "5H")
          boardCard1 = card15;
      else if (string.substring(0,2) === "5S")
          boardCard1 = card16;
      else if (string.substring(0,2) === "6C")
          boardCard1 = card17;
      else if (string.substring(0,2) === "6D")
          boardCard1 = card18;
      else if (string.substring(0,2) === "6H")
          boardCard1 = card19;
      else if (string.substring(0,2) === "6S")
          boardCard1 = card20;
      else if (string.substring(0,2) === "7C")
          boardCard1 = card21;
      else if (string.substring(0,2) === "7D")
          boardCard1 = card22;
      else if (string.substring(0,2) === "7H")
          boardCard1 = card23;
      else if (string.substring(0,2) === "7S")
          boardCard1 = card24;
      else if (string.substring(0,2) === "8C")
          boardCard1 = card25;
      else if (string.substring(0,2) === "8D")
          boardCard1 = card26;
      else if (string.substring(0,2) === "8H")
          boardCard1 = card27;
      else if (string.substring(0,2) === "8S")
          boardCard1 = card28;
      else if (string.substring(0,2) === "9C")
          boardCard1 = card29;
      else if (string.substring(0,2) === "9D")
          boardCard1 = card30;
      else if (string.substring(0,2) === "9H")
          boardCard1 = card31;
      else if (string.substring(0,2) === "9S")
          boardCard1 = card32;
      else if (string.substring(0,2) === "TC")
          boardCard1 = card33;
      else if (string.substring(0,2) === "TD")
          boardCard1 = card34;
      else if (string.substring(0,2) === "TH")
          boardCard1 = card35;
      else if (string.substring(0,2) === "TS")
          boardCard1 = card36;
      else if (string.substring(0,2) === "JC")
          boardCard1 = card37;
      else if (string.substring(0,2) === "JD")
          boardCard1 = card38;
      else if (string.substring(0,2) === "JH")
          boardCard1 = card39;
      else if (string.substring(0,2) === "JS")
          boardCard1 = card40;
      else if (string.substring(0,2) === "QC")
          boardCard1 = card41;
      else if (string.substring(0,2) === "QD")
          boardCard1 = card42;
      else if (string.substring(0,2) === "QH")
          boardCard1 = card43;
      else if (string.substring(0,2) === "QS")
          boardCard1 = card44;
      else if (string.substring(0,2) === "KC")
          boardCard1 = card45;
      else if (string.substring(0,2) === "KD")
          boardCard1 = card46;
      else if (string.substring(0,2) === "KH")
          boardCard1 = card47;
      else if (string.substring(0,2) === "KS")
          boardCard1 = card48;
      else if (string.substring(0,2) === "AC")
          boardCard1 = card49;
      else if (string.substring(0,2) === "AD")
          boardCard1 = card50;
      else if (string.substring(0,2) === "AH")
          boardCard1 = card51;
      else if (string.substring(0,2) === "AS")
          boardCard1 = card52;

      if (string.substring(2,4) === "2C")
        boardCard2 = card1;
      else if (string.substring(2,4) === "2D")
        boardCard2 = card2;
      else if (string.substring(2,4) === "2H")
          boardCard2 = card3;
      else if (string.substring(2,4) === "2S")
          boardCard2 = card4;
      else if (string.substring(2,4) === "3C")
          boardCard2 = card5;
      else if (string.substring(2,4) === "3D")
          boardCard2 = card6;
      else if (string.substring(2,4) === "3H")
          boardCard2 = card7;
      else if (string.substring(2,4) === "3S")
          boardCard2 = card8;
      else if (string.substring(2,4) === "4C")
          boardCard2 = card9;
      else if (string.substring(2,4) === "4D")
          boardCard2 = card10;
      else if (string.substring(2,4) === "4H")
          boardCard2 = card11;
      else if (string.substring(2,4) === "4S")
          boardCard2 = card12;
      else if (string.substring(2,4) === "5C")
          boardCard2 = card13;
      else if (string.substring(2,4) === "5D")
          boardCard2 = card14;
      else if (string.substring(2,4) === "5H")
          boardCard2 = card15;
      else if (string.substring(2,4) === "5S")
          boardCard2 = card16;
      else if (string.substring(2,4) === "6C")
          boardCard2 = card17;
      else if (string.substring(2,4) === "6D")
          boardCard2 = card18;
      else if (string.substring(2,4) === "6H")
          boardCard2 = card19;
      else if (string.substring(2,4) === "6S")
          boardCard2 = card20;
      else if (string.substring(2,4) === "7C")
          boardCard2 = card21;
      else if (string.substring(2,4) === "7D")
          boardCard2 = card22;
      else if (string.substring(2,4) === "7H")
          boardCard2 = card23;
      else if (string.substring(2,4) === "7S")
          boardCard2 = card24;
      else if (string.substring(2,4) === "8C")
          boardCard2 = card25;
      else if (string.substring(2,4) === "8D")
          boardCard2 = card26;
      else if (string.substring(2,4) === "8H")
          boardCard2 = card27;
      else if (string.substring(2,4) === "8S")
          boardCard2 = card28;
      else if (string.substring(2,4) === "9C")
          boardCard2 = card29;
      else if (string.substring(2,4) === "9D")
          boardCard2 = card30;
      else if (string.substring(2,4) === "9H")
          boardCard2 = card31;
      else if (string.substring(2,4) === "9S")
          boardCard2 = card32;
      else if (string.substring(2,4) === "TC")
          boardCard2 = card33;
      else if (string.substring(2,4) === "TD")
          boardCard2 = card34;
      else if (string.substring(2,4) === "TH")
          boardCard2 = card35;
      else if (string.substring(2,4) === "TS")
          boardCard2 = card36;
      else if (string.substring(2,4) === "JC")
          boardCard2 = card37;
      else if (string.substring(2,4) === "JD")
          boardCard2 = card38;
      else if (string.substring(2,4) === "JH")
          boardCard2 = card39;
      else if (string.substring(2,4) === "JS")
          boardCard2 = card40;
      else if (string.substring(2,4) === "QC")
          boardCard2 = card41;
      else if (string.substring(2,4) === "QD")
          boardCard2 = card42;
      else if (string.substring(2,4) === "QH")
          boardCard2 = card43;
      else if (string.substring(2,4) === "QS")
          boardCard2 = card44;
      else if (string.substring(2,4) === "KC")
          boardCard2 = card45;
      else if (string.substring(2,4) === "KD")
          boardCard2 = card46;
      else if (string.substring(2,4) === "KH")
          boardCard2 = card47;
      else if (string.substring(2,4) === "KS")
          boardCard2 = card48;
      else if (string.substring(2,4) === "AC")
          boardCard2 = card49;
      else if (string.substring(2,4) === "AD")
          boardCard2 = card50;
      else if (string.substring(2,4) === "AH")
          boardCard2 = card51;
      else if (string.substring(2,4) === "AS")
          boardCard2 = card52;


      if (string.substring(4,6) === "2C")
        boardCard3 = card1;
      else if (string.substring(4,6) === "2D")
        boardCard3 = card2;
      else if (string.substring(4,6) === "2H")
          boardCard3 = card3;
      else if (string.substring(4,6) === "2S")
          boardCard3 = card4;
      else if (string.substring(4,6) === "3C")
          boardCard3 = card5;
      else if (string.substring(4,6) === "3D")
          boardCard3 = card6;
      else if (string.substring(4,6) === "3H")
          boardCard3 = card7;
      else if (string.substring(4,6) === "3S")
          boardCard3 = card8;
      else if (string.substring(4,6) === "4C")
          boardCard3 = card9;
      else if (string.substring(4,6) === "4D")
          boardCard3 = card10;
      else if (string.substring(4,6) === "4H")
          boardCard3 = card11;
      else if (string.substring(4,6) === "4S")
          boardCard3 = card12;
      else if (string.substring(4,6) === "5C")
          boardCard3 = card13;
      else if (string.substring(4,6) === "5D")
          boardCard3 = card14;
      else if (string.substring(4,6) === "5H")
          boardCard3 = card15;
      else if (string.substring(4,6) === "5S")
          boardCard3 = card16;
      else if (string.substring(4,6) === "6C")
          boardCard3 = card17;
      else if (string.substring(4,6) === "6D")
          boardCard3 = card18;
      else if (string.substring(4,6) === "6H")
          boardCard3 = card19;
      else if (string.substring(4,6) === "6S")
          boardCard3 = card20;
      else if (string.substring(4,6) === "7C")
          boardCard3 = card21;
      else if (string.substring(4,6) === "7D")
          boardCard3 = card22;
      else if (string.substring(4,6) === "7H")
          boardCard3 = card23;
      else if (string.substring(4,6) === "7S")
          boardCard3 = card24;
      else if (string.substring(4,6) === "8C")
          boardCard3 = card25;
      else if (string.substring(4,6) === "8D")
          boardCard3 = card26;
      else if (string.substring(4,6) === "8H")
          boardCard3 = card27;
      else if (string.substring(4,6) === "8S")
          boardCard3 = card28;
      else if (string.substring(4,6) === "9C")
          boardCard3 = card29;
      else if (string.substring(4,6) === "9D")
          boardCard3 = card30;
      else if (string.substring(4,6) === "9H")
          boardCard3 = card31;
      else if (string.substring(4,6) === "9S")
          boardCard3 = card32;
      else if (string.substring(4,6) === "TC")
          boardCard3 = card33;
      else if (string.substring(4,6) === "TD")
          boardCard3 = card34;
      else if (string.substring(4,6) === "TH")
          boardCard3 = card35;
      else if (string.substring(4,6) === "TS")
          boardCard3 = card36;
      else if (string.substring(4,6) === "JC")
          boardCard3 = card37;
      else if (string.substring(4,6) === "JD")
          boardCard3 = card38;
      else if (string.substring(4,6) === "JH")
          boardCard3 = card39;
      else if (string.substring(4,6) === "JS")
          boardCard3 = card40;
      else if (string.substring(4,6) === "QC")
          boardCard3 = card41;
      else if (string.substring(4,6) === "QD")
          boardCard3 = card42;
      else if (string.substring(4,6) === "QH")
          boardCard3 = card43;
      else if (string.substring(4,6) === "QS")
          boardCard3 = card44;
      else if (string.substring(4,6) === "KC")
          boardCard3 = card45;
      else if (string.substring(4,6) === "KD")
          boardCard3 = card46;
      else if (string.substring(4,6) === "KH")
          boardCard3 = card47;
      else if (string.substring(4,6) === "KS")
          boardCard3 = card48;
      else if (string.substring(4,6) === "AC")
          boardCard3 = card49;
      else if (string.substring(4,6) === "AD")
          boardCard3 = card50;
      else if (string.substring(4,6) === "AH")
          boardCard3 = card51;
      else if (string.substring(4,6) === "AS")
          boardCard3 = card52;
          boardCard4 = card;
          boardCard5 = card;

      }

      if (string.length >= 8) {
        if (string.substring(6,8) === "2C")
          boardCard4 = card1;
        else if (string.substring(6,8) === "2D")
          boardCard4 = card2;
        else if (string.substring(6,8) === "2H")
            boardCard4 = card3;
        else if (string.substring(6,8) === "2S")
            boardCard4 = card4;
        else if (string.substring(6,8) === "3C")
            boardCard4 = card5;
        else if (string.substring(6,8) === "3D")
            boardCard4 = card6;
        else if (string.substring(6,8) === "3H")
            boardCard4 = card7;
        else if (string.substring(6,8) === "3S")
            boardCard4 = card8;
        else if (string.substring(6,8) === "4C")
            boardCard4 = card9;
        else if (string.substring(6,8) === "4D")
            boardCard4 = card10;
        else if (string.substring(6,8) === "4H")
            boardCard4 = card11;
        else if (string.substring(6,8) === "4S")
            boardCard4 = card12;
        else if (string.substring(6,8) === "5C")
            boardCard4 = card13;
        else if (string.substring(6,8) === "5D")
            boardCard4 = card14;
        else if (string.substring(6,8) === "5H")
            boardCard4 = card15;
        else if (string.substring(6,8) === "5S")
            boardCard4 = card16;
        else if (string.substring(6,8) === "6C")
            boardCard4 = card17;
        else if (string.substring(6,8) === "6D")
            boardCard4 = card18;
        else if (string.substring(6,8) === "6H")
            boardCard4 = card19;
        else if (string.substring(6,8) === "6S")
            boardCard4 = card20;
        else if (string.substring(6,8) === "7C")
            boardCard4 = card21;
        else if (string.substring(6,8) === "7D")
            boardCard4 = card22;
        else if (string.substring(6,8) === "7H")
            boardCard4 = card23;
        else if (string.substring(6,8) === "7S")
            boardCard4 = card24;
        else if (string.substring(6,8) === "8C")
            boardCard4 = card25;
        else if (string.substring(6,8) === "8D")
            boardCard4 = card26;
        else if (string.substring(6,8) === "8H")
            boardCard4 = card27;
        else if (string.substring(6,8) === "8S")
            boardCard4 = card28;
        else if (string.substring(6,8) === "9C")
            boardCard4 = card29;
        else if (string.substring(6,8) === "9D")
            boardCard4 = card30;
        else if (string.substring(6,8) === "9H")
            boardCard4 = card31;
        else if (string.substring(6,8) === "9S")
            boardCard4 = card32;
        else if (string.substring(6,8) === "TC")
            boardCard4 = card33;
        else if (string.substring(6,8) === "TD")
            boardCard4 = card34;
        else if (string.substring(6,8) === "TH")
            boardCard4 = card35;
        else if (string.substring(6,8) === "TS")
            boardCard4 = card36;
        else if (string.substring(6,8) === "JC")
            boardCard4 = card37;
        else if (string.substring(6,8) === "JD")
            boardCard4 = card38;
        else if (string.substring(6,8) === "JH")
            boardCard4 = card39;
        else if (string.substring(6,8) === "JS")
            boardCard4 = card40;
        else if (string.substring(6,8) === "QC")
            boardCard4 = card41;
        else if (string.substring(6,8) === "QD")
            boardCard4 = card42;
        else if (string.substring(6,8) === "QH")
            boardCard4 = card43;
        else if (string.substring(6,8) === "QS")
            boardCard4 = card44;
        else if (string.substring(6,8) === "KC")
            boardCard4 = card45;
        else if (string.substring(6,8) === "KD")
            boardCard4 = card46;
        else if (string.substring(6,8) === "KH")
            boardCard4 = card47;
        else if (string.substring(6,8) === "KS")
            boardCard4 = card48;
        else if (string.substring(6,8) === "AC")
            boardCard4 = card49;
        else if (string.substring(6,8) === "AD")
            boardCard4 = card50;
        else if (string.substring(6,8) === "AH")
            boardCard4 = card51;
        else if (string.substring(6,8) === "AS")
            boardCard4 = card52;

      } if (string.length === 10) {
        if (string.substring(8,10) === "2C")
          boardCard5 = card1;
        else if (string.substring(8,10) === "2D")
          boardCard5 = card2;
        else if (string.substring(8,10) === "2H")
            boardCard5 = card3;
        else if (string.substring(8,10) === "2S")
            boardCard5 = card4;
        else if (string.substring(8,10) === "3C")
            boardCard5 = card5;
        else if (string.substring(8,10) === "3D")
            boardCard5 = card6;
        else if (string.substring(8,10) === "3H")
            boardCard5 = card7;
        else if (string.substring(8,10) === "3S")
            boardCard5 = card8;
        else if (string.substring(8,10) === "4C")
            boardCard5 = card9;
        else if (string.substring(8,10) === "4D")
            boardCard5 = card10;
        else if (string.substring(8,10) === "4H")
            boardCard5 = card11;
        else if (string.substring(8,10) === "4S")
            boardCard5 = card12;
        else if (string.substring(8,10) === "5C")
            boardCard5 = card13;
        else if (string.substring(8,10) === "5D")
            boardCard5 = card14;
        else if (string.substring(8,10) === "5H")
            boardCard5 = card15;
        else if (string.substring(8,10) === "5S")
            boardCard5 = card16;
        else if (string.substring(8,10) === "6C")
            boardCard5 = card17;
        else if (string.substring(8,10) === "6D")
            boardCard5 = card18;
        else if (string.substring(8,10) === "6H")
            boardCard5 = card19;
        else if (string.substring(8,10) === "6S")
            boardCard5 = card20;
        else if (string.substring(8,10) === "7C")
            boardCard5 = card21;
        else if (string.substring(8,10) === "7D")
            boardCard5 = card22;
        else if (string.substring(8,10) === "7H")
            boardCard5 = card23;
        else if (string.substring(8,10) === "7S")
            boardCard5 = card24;
        else if (string.substring(8,10) === "8C")
            boardCard5 = card25;
        else if (string.substring(8,10) === "8D")
            boardCard5 = card26;
        else if (string.substring(8,10) === "8H")
            boardCard5 = card27;
        else if (string.substring(8,10) === "8S")
            boardCard5 = card28;
        else if (string.substring(8,10) === "9C")
            boardCard5 = card29;
        else if (string.substring(8,10) === "9D")
            boardCard5 = card30;
        else if (string.substring(8,10) === "9H")
            boardCard5 = card31;
        else if (string.substring(8,10) === "9S")
            boardCard5 = card32;
        else if (string.substring(8,10) === "TC")
            boardCard5 = card33;
        else if (string.substring(8,10) === "TD")
            boardCard5 = card34;
        else if (string.substring(8,10) === "TH")
            boardCard5 = card35;
        else if (string.substring(8,10) === "TS")
            boardCard5 = card36;
        else if (string.substring(8,10) === "JC")
            boardCard5 = card37;
        else if (string.substring(8,10) === "JD")
            boardCard5 = card38;
        else if (string.substring(8,10) === "JH")
            boardCard5 = card39;
        else if (string.substring(8,10) === "JS")
            boardCard5 = card40;
        else if (string.substring(8,10) === "QC")
            boardCard5 = card41;
        else if (string.substring(8,10) === "QD")
            boardCard5 = card42;
        else if (string.substring(8,10) === "QH")
            boardCard5 = card43;
        else if (string.substring(8,10) === "QS")
            boardCard5 = card44;
        else if (string.substring(8,10) === "KC")
            boardCard5 = card45;
        else if (string.substring(8,10) === "KD")
            boardCard5 = card46;
        else if (string.substring(8,10) === "KH")
            boardCard5 = card47;
        else if (string.substring(8,10) === "KS")
            boardCard5 = card48;
        else if (string.substring(8,10) === "AC")
            boardCard5 = card49;
        else if (string.substring(8,10) === "AD")
            boardCard5 = card50;
        else if (string.substring(8,10) === "AH")
            boardCard5 = card51;
        else if (string.substring(8,10) === "AS")
            boardCard5 = card52;
      }
    }

      } else if (set === 3) { //Updates AI's cards
    if(string === "") {
      botCard1 = card;
      botCard2 = card;
    } else {

      if (string.substring(0,2) === "2C")
        botCard1 = card1;
      else if (string.substring(0,2) === "2D")
        botCard1 = card2;
      else if (string.substring(0,2) === "2H")
          botCard1 = card3;
      else if (string.substring(0,2) === "2S")
          botCard1 = card4;
      else if (string.substring(0,2) === "3C")
          botCard1 = card5;
      else if (string.substring(0,2) === "3D")
          botCard1 = card6;
      else if (string.substring(0,2) === "3H")
          botCard1 = card7;
      else if (string.substring(0,2) === "3S")
          botCard1 = card8;
      else if (string.substring(0,2) === "4C")
          botCard1 = card9;
      else if (string.substring(0,2) === "4D")
          botCard1 = card10;
      else if (string.substring(0,2) === "4H")
          botCard1 = card11;
      else if (string.substring(0,2) === "4S")
          botCard1 = card12;
      else if (string.substring(0,2) === "5C")
          botCard1 = card13;
      else if (string.substring(0,2) === "5D")
          botCard1 = card14;
      else if (string.substring(0,2) === "5H")
          botCard1 = card15;
      else if (string.substring(0,2) === "5S")
          botCard1 = card16;
      else if (string.substring(0,2) === "6C")
          botCard1 = card17;
      else if (string.substring(0,2) === "6D")
          botCard1 = card18;
      else if (string.substring(0,2) === "6H")
          botCard1 = card19;
      else if (string.substring(0,2) === "6S")
          botCard1 = card20;
      else if (string.substring(0,2) === "7C")
          botCard1 = card21;
      else if (string.substring(0,2) === "7D")
          botCard1 = card22;
      else if (string.substring(0,2) === "7H")
          botCard1 = card23;
      else if (string.substring(0,2) === "7S")
          botCard1 = card24;
      else if (string.substring(0,2) === "8C")
          botCard1 = card25;
      else if (string.substring(0,2) === "8D")
          botCard1 = card26;
      else if (string.substring(0,2) === "8H")
          botCard1 = card27;
      else if (string.substring(0,2) === "8S")
          botCard1 = card28;
      else if (string.substring(0,2) === "9C")
          botCard1 = card29;
      else if (string.substring(0,2) === "9D")
          botCard1 = card30;
      else if (string.substring(0,2) === "9H")
          botCard1 = card31;
      else if (string.substring(0,2) === "9S")
          botCard1 = card32;
      else if (string.substring(0,2) === "TC")
          botCard1 = card33;
      else if (string.substring(0,2) === "TD")
          botCard1 = card34;
      else if (string.substring(0,2) === "TH")
          botCard1 = card35;
      else if (string.substring(0,2) === "TS")
          botCard1 = card36;
      else if (string.substring(0,2) === "JC")
          botCard1 = card37;
      else if (string.substring(0,2) === "JD")
          botCard1 = card38;
      else if (string.substring(0,2) === "JH")
          botCard1 = card39;
      else if (string.substring(0,2) === "JS")
          botCard1 = card40;
      else if (string.substring(0,2) === "QC")
          botCard1 = card41;
      else if (string.substring(0,2) === "QD")
          botCard1 = card42;
      else if (string.substring(0,2) === "QH")
          botCard1 = card43;
      else if (string.substring(0,2) === "QS")
          botCard1 = card44;
      else if (string.substring(0,2) === "KC")
          botCard1 = card45;
      else if (string.substring(0,2) === "KD")
          botCard1 = card46;
      else if (string.substring(0,2) === "KH")
          botCard1 = card47;
      else if (string.substring(0,2) === "KS")
          botCard1 = card48;
      else if (string.substring(0,2) === "AC")
          botCard1 = card49;
      else if (string.substring(0,2) === "AD")
          botCard1 = card50;
      else if (string.substring(0,2) === "AH")
          botCard1 = card51;
      else if (string.substring(0,2) === "AS")
          botCard1 = card52;

          if (string.substring(2,4) === "2C")
            botCard2 = card1;
          else if (string.substring(2,4) === "2D")
            botCard2 = card2;
          else if (string.substring(2,4) === "2H")
              botCard2 = card3;
          else if (string.substring(2,4) === "2S")
              botCard2 = card4;
          else if (string.substring(2,4) === "3C")
              botCard2 = card5;
          else if (string.substring(2,4) === "3D")
              botCard2 = card6;
          else if (string.substring(2,4) === "3H")
              botCard2 = card7;
          else if (string.substring(2,4) === "3S")
              botCard2 = card8;
          else if (string.substring(2,4) === "4C")
              botCard2 = card9;
          else if (string.substring(2,4) === "4D")
              botCard2 = card10;
          else if (string.substring(2,4) === "4H")
              botCard2 = card11;
          else if (string.substring(2,4) === "4S")
              botCard2 = card12;
          else if (string.substring(2,4) === "5C")
              botCard2 = card13;
          else if (string.substring(2,4) === "5D")
              botCard2 = card14;
          else if (string.substring(2,4) === "5H")
              botCard2 = card15;
          else if (string.substring(2,4) === "5S")
              botCard2 = card16;
          else if (string.substring(2,4) === "6C")
              botCard2 = card17;
          else if (string.substring(2,4) === "6D")
              botCard2 = card18;
          else if (string.substring(2,4) === "6H")
              botCard2 = card19;
          else if (string.substring(2,4) === "6S")
              botCard2 = card20;
          else if (string.substring(2,4) === "7C")
              botCard2 = card21;
          else if (string.substring(2,4) === "7D")
              botCard2 = card22;
          else if (string.substring(2,4) === "7H")
              botCard2 = card23;
          else if (string.substring(2,4) === "7S")
              botCard2 = card24;
          else if (string.substring(2,4) === "8C")
              botCard2 = card25;
          else if (string.substring(2,4) === "8D")
              botCard2 = card26;
          else if (string.substring(2,4) === "8H")
              botCard2 = card27;
          else if (string.substring(2,4) === "8S")
              botCard2 = card28;
          else if (string.substring(2,4) === "9C")
              botCard2 = card29;
          else if (string.substring(2,4) === "9D")
              botCard2 = card30;
          else if (string.substring(2,4) === "9H")
              botCard2 = card31;
          else if (string.substring(2,4) === "9S")
              botCard2 = card32;
          else if (string.substring(2,4) === "TC")
              botCard2 = card33;
          else if (string.substring(2,4) === "TD")
              botCard2 = card34;
          else if (string.substring(2,4) === "TH")
              botCard2 = card35;
          else if (string.substring(2,4) === "TS")
              botCard2 = card36;
          else if (string.substring(2,4) === "JC")
              botCard2 = card37;
          else if (string.substring(2,4) === "JD")
              botCard2 = card38;
          else if (string.substring(2,4) === "JH")
              botCard2 = card39;
          else if (string.substring(2,4) === "JS")
              botCard2 = card40;
          else if (string.substring(2,4) === "QC")
              botCard2 = card41;
          else if (string.substring(2,4) === "QD")
              botCard2 = card42;
          else if (string.substring(2,4) === "QH")
              botCard2 = card43;
          else if (string.substring(2,4) === "QS")
              botCard2 = card44;
          else if (string.substring(2,4) === "KC")
              botCard2 = card45;
          else if (string.substring(2,4) === "KD")
              botCard2 = card46;
          else if (string.substring(2,4) === "KH")
              botCard2 = card47;
          else if (string.substring(2,4) === "KS")
              botCard2 = card48;
          else if (string.substring(2,4) === "AC")
              botCard2 = card49;
          else if (string.substring(2,4) === "AD")
              botCard2 = card50;
          else if (string.substring(2,4) === "AH")
              botCard2 = card51;
          else if (string.substring(2,4) === "AS")
              botCard2 = card52;
        }
  }
}

cardDisplayer(1, hand);
cardDisplayer(2, board);
cardDisplayer(3, botHand);
export default function App() {
  return (
    <div style ={{ backgroundImage: `url(${table})`,
      backgroundSize: 'cover',
      }}>

    <Box p = {1} display="flex" alignItems = "center" justifyContent = "center">
      <img src={botCard1} width ="60" height = "90" classname="test-card" alt="card"/>
      <img src={botCard2} width ="60" height = "90" classname="test-card" alt="card"/>
    </Box>
    <Box p = {1} display="flex" alignItems = "center" justifyContent = "center">
      <img src={boardCard1} width ="60" height = "90" classname="test-card" alt="card"/>
      <img src={boardCard2} width ="60" height = "90" classname="test-card" alt="card"/>
      <img src={boardCard3} width ="60" height = "90" classname="test-card" alt="card"/>
      <img src={boardCard4} width ="60" height = "90" classname="test-card" alt="card"/>
      <img src={boardCard5} width ="60" height = "90" classname="test-card" alt="card"/>
    </Box>
    <Box p = {1} display="flex" alignItems = "center" justifyContent = "center">
      <img src={playerCard1} width ="60" height = "90" classname="test-card" alt="card"/>
      <img src={playerCard2} width ="60" height = "90" classname="test-card" alt="card"/>
    </Box>

    <Box m = {4} display="flex" alignItems = "Left" justifyContent = "Left">
      Outcome: {resultText}
    </Box>

    <Box p = {1} display="flex" alignItems="center" justifyContent="center">
    <Button onClick = {nextHand} variant = "contained">
      Next Hand </Button>
    <Button onClick = {fold} variant = "contained">
      Fold</Button>
    <Button onClick = {check} variant = "contained">
      Check</Button>
    <Button onClick = {call} variant = "contained">
      Call</Button>
    <Button onClick = {peek} variant = "contained">
      Peek</Button>
    </Box>

    <Box p = {1} display="flex" alignItems="center" justifyContent="center">
    <Button onClick = {minBet} variant = "contained">
      Min Bet</Button>
    <Button onClick = {halfPot} variant = "contained">
      Bet Half Pot</Button>
    <Button onClick = {fullPot} variant = "contained">
      Bet Pot</Button>
    <Button onClick = {allIn} variant = "contained">
      All In</Button>
    <Button onClick = {specificBet} variant = "contained">
      Bet</Button>
    </Box>

    <Box p = {1} display = "flex" alignItems="center" justifyContent="center">
    <TextField id="Bet-Entry" label = "Bet Amount" variant = "filled" />
    </Box>
      </div>
  );
}
