import { Typography } from "@material-ui/core";
import React from "react";
import { Route, Switch } from "react-router-dom";
import NavBar from "./NavBar";
import LandingPage from "./pages/LandingPage";

function Router() {
  return (
    <>
      <NavBar />
      <Switch>
        <Route exact path="/">
          <LandingPage />
        </Route>
        <Route exact path="/bot">
          <h1>welcome</h1>
        </Route>
      </Switch>
    </>
  );
}

export default Router;
