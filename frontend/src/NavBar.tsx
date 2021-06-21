import {
  AppBar,
  Button,
  IconButton,
  makeStyles,
  Toolbar,
  Typography,
} from "@material-ui/core";
import MenuIcon from "@material-ui/icons/Menu";
import React from "react";

const useStyles = makeStyles((theme) => ({
  root: {
    flexGrow: 1,
  },
  menuButton: {
    marginRight: theme.spacing(2),
  },
  title: {
    // flexGrow: 1,
  },
}));

export default function NavBar() {
  const classes = useStyles();

  // TODO: use a navbar when i have authentication stuff in place
  //   return (
  //     <AppBar position="static">
  //       <Toolbar>
  //         <Typography variant="h6" className={classes.title}>
  //           Narc
  //         </Typography>
  //         <Button color="inherit">Login</Button>
  //       </Toolbar>
  //     </AppBar>
  //   );
  return null;
}
