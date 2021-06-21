import { Button, Grid, makeStyles, Typography } from "@material-ui/core";

const useStyles = makeStyles({
  root: {
    flexGrow: 1,
    minHeight: "20rem",
  },
  title: {
    textAlign: "center",
  },
  subtitle: {
    textAlign: "center",
    marginLeft: "1rem",
    marginRight: "1rem",
  },
  spacer: {
    minHeight: "1rem",
  },
  getStartedContainer: {
    marginRight: "1rem",
  },
  aboutContainer: {
    marginLeft: "1rem",
  },
  getStartedButton: {
    minWidth: "8rem",
  },
  aboutButton: {
    minWidth: "8rem",
  },
});

export default function LandingPage() {
  const classes = useStyles();

  return (
    <Grid
      className={classes.root}
      container
      //   alignItems="center"
      direction="column"
    >
      <Grid item xs />
      <Grid item container alignItems="center" direction="row">
        <Grid item xs />
        <Grid className={classes.title} item>
          <Typography variant="h3" component="h2">
            Narc
          </Typography>
        </Grid>
        <Grid item xs />
      </Grid>
      <Grid className={classes.spacer} item />
      <Grid item container alignItems="center" direction="row">
        <Grid item xs />
        <Grid className={classes.subtitle} item>
          <Typography variant="h6" component="h6">
            {/* TODO: have a standard catch phrase */}
            Narc is a Discord Bot for letting users quickly inform moderators
            about wrongdoings.
          </Typography>
        </Grid>
        <Grid item xs />
      </Grid>
      <Grid className={classes.spacer} item />
      <Grid item container alignItems="center" direction="row">
        <Grid item xs />
        <Grid className={classes.getStartedContainer} item>
          <Button
            className={classes.getStartedButton}
            variant="contained"
            color="secondary"
          >
            Get Started
          </Button>
        </Grid>
        <Grid className={classes.aboutContainer} item>
          <Button
            className={classes.aboutButton}
            variant="contained"
            color="secondary"
          >
            About
          </Button>
        </Grid>
        <Grid item xs />
      </Grid>
      <Grid item xs />
    </Grid>
  );
}
