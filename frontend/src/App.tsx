import React from "react";
import "./App.css";
import { Route, Switch } from "react-router-dom";

function App() {
  return (
    <Switch>
      <Route exact path="/">
        <h1>
          hi, <a href="/bot">click</a>
        </h1>
      </Route>
      <Route exact path="/bot">
        <h1>welcome</h1>
      </Route>
    </Switch>
  );
}

export default App;
