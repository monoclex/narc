import React, { useMemo } from "react";
import ReactDOM from "react-dom";
import { BrowserRouter } from "react-router-dom";
import "./index.css";
import App from "./App";
import reportWebVitals from "./reportWebVitals";
import {
  createMuiTheme,
  CssBaseline,
  ThemeProvider,
  useMediaQuery,
} from "@material-ui/core";
import { deepPurple, indigo } from "@material-ui/core/colors";
import { RecoilRoot } from "recoil";
import { SnackbarProvider } from "notistack";

ReactDOM.render(
  <Boilerplate>
    <App />
  </Boilerplate>,
  document.getElementById("root")
);

// If you want to start measuring performance in your app, pass a function
// to log results (for example: reportWebVitals(console.log))
// or send to an analytics endpoint. Learn more: https://bit.ly/CRA-vitals
reportWebVitals(console.log);

interface BoilerplateProps {
  children: React.ReactChild;
}

function Boilerplate({ children }: BoilerplateProps) {
  const prefersLightMode = useMediaQuery("prefers-color-scheme: light");

  const theme = useMemo(
    () =>
      createMuiTheme({
        palette: {
          type: prefersLightMode ? "light" : "dark",
          primary: indigo,
          secondary: deepPurple,
        },
      }),
    [prefersLightMode]
  );

  return (
    <React.StrictMode>
      <BrowserRouter>
        <RecoilRoot>
          <SnackbarProvider>
            <ThemeProvider theme={theme}>
              <CssBaseline />
              {children}
            </ThemeProvider>
          </SnackbarProvider>
        </RecoilRoot>
      </BrowserRouter>
    </React.StrictMode>
  );
}
