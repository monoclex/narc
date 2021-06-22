///////////////////////////////////////////////////////////////////////////////
//
// TODO: this code is HELLA UGLY
// later imma just copy https://discord.com/ or https://groovy.bot/ for the
// design, but for now, please do know that YES. I AGREE. IT'S UGLY.
//
///////////////////////////////////////////////////////////////////////////////

import React from "react";
import {
  Button,
  Grid,
  makeStyles,
  Typography,
  useMediaQuery,
  useTheme,
} from "@material-ui/core";
import clsx from "clsx";
import reportMessageWebm from "../assets/report_message.webm";
import incomingReportWebm from "../assets/incoming_report.webm";

const INVITE_DISCORD_URL =
  "https://discord.com/api/oauth2/authorize?client_id=838665329604427777&permissions=125952&scope=bot";

const REPORT_MESSAGE_VIDEO_WIDTH_PIXELS = 484;

const useStyles = makeStyles((theme) => ({
  section: {
    flexGrow: 1,
    minHeight: "20rem",
  },
  sectionLeftRightPadding: {
    paddingLeft: "1rem",
    paddingRight: "1rem",
  },
  centerText: {
    textAlign: "center",
  },
  button: {
    margin: "1rem",
    // the text is the same size so don't need to do this
    // minWidth: "9rem",
    color: "#000 !important",
  },
  stupidHack: {
    // marginRight: "0.3em",
    marginRight: "0.05em",
  },
  spacedText: {
    lineHeight: "2em",
  },
  cushionedVert: {
    marginTop: "1em",
    marginBottom: "1em",
  },
  cusionedHorz: {
    marginLeft: "4em",
    marginRight: "4em",
  },
  buttonText: {
    color: "black",
  },
  openSource: {
    maxWidth: `${REPORT_MESSAGE_VIDEO_WIDTH_PIXELS}px`,
  },
  sectionSpacing: {
    marginBottom: "7.5em",
  },
}));

function HeroHeader() {
  const classes = useStyles();

  return (
    <Grid
      className={clsx(
        classes.section,
        classes.centerText,
        classes.sectionLeftRightPadding
      )}
      container
      justify="center"
      direction="column"
    >
      <Grid item container justify="center">
        <Typography className={classes.stupidHack} variant="h3" component="h2">
          Narc
        </Typography>
      </Grid>
      <Grid item container justify="center">
        <Typography variant="h6" component="h6">
          {/* TODO: have a standard catch phrase */}
          Narc is a Discord bot for letting users quickly inform moderators
          about wrongdoings.
        </Typography>
      </Grid>
      <Grid item container justify="center" alignItems="center" direction="row">
        <Grid item>
          <Button
            className={classes.button}
            variant="contained"
            color="secondary"
            href={INVITE_DISCORD_URL}
          >
            <span className={classes.buttonText}>Invite Now</span>
          </Button>
        </Grid>
        {/* <Grid item>
          <Button
            className={classes.button}
            variant="contained"
            color="secondary"
          >
            <span className={classes.buttonText}>Learn More</span>
          </Button>
        </Grid> */}
      </Grid>
    </Grid>
  );
}

const useTemp = makeStyles({
  valuesText: {
    maxWidth: "30rem",
  },
  noLeftRightPadding: {
    paddingLeft: "0 !important",
    paddingRight: "0 !important",
  },
  video: {
    marginLeft: "1rem",
  },
  videoRev: {
    marginRight: "1rem",
  },
  maxWidthReportMessageVideo: {
    maxWidth: "",
  },
  fixLineHeight: {
    lineHeight: 0,
    // if the video goes off screen we don't care
    overflowX: "hidden",
  },
  msgLeft: {
    width: `${REPORT_MESSAGE_VIDEO_WIDTH_PIXELS / 1.2}px`,
    marginRight: "1rem",
    marginTop: "1rem",
  },
  msgRight: {
    width: `${REPORT_MESSAGE_VIDEO_WIDTH_PIXELS / 1.2}px`,
    marginLeft: "1rem",
    marginTop: "1rem",
  },
});

type ValueProps = VideoValueProps | ChildrenValueProps;

interface VideoValueProps {
  title: string;
  message: string;
  videoUrl: string;
  isReversed?: boolean;
  spacingBelow?: boolean;
  children?: undefined;
}

interface ChildrenValueProps {
  title: string;
  message: string;
  videoUrl?: undefined;
  isReversed?: boolean;
  spacingBelow?: boolean;
  children: React.ReactChild;
}

function Value({
  title,
  message,
  videoUrl,
  isReversed,
  children,
  spacingBelow,
}: ValueProps) {
  const classes = useStyles();
  const c2 = useTemp();
  const theme = useTheme();
  const isDesktop = useMediaQuery(theme.breakpoints.up("md"));

  function Message() {
    return (
      <>
        <Typography className={classes.spacedText} variant="h5" component="h4">
          {title}
        </Typography>
        <Typography variant="subtitle2" component="h6">
          {message}
        </Typography>
      </>
    );
  }

  if (isDesktop) {
    return (
      <Grid
        className={clsx(
          classes.section,
          spacingBelow && classes.sectionSpacing
        )}
        container
        direction={isReversed ? "row-reverse" : "row"}
      >
        <Grid
          item
          md={6}
          container
          justify={isReversed ? "flex-start" : "flex-end"}
        >
          <Grid className={isReversed ? c2.msgRight : c2.msgLeft} item>
            <Message />
          </Grid>
        </Grid>
        <Grid
          className={c2.fixLineHeight}
          item
          md={6}
          container
          justify={isReversed ? "flex-end" : undefined}
        >
          <Grid item>
            {!!videoUrl ? (
              <video
                className={isReversed ? c2.videoRev : c2.video}
                autoPlay
                loop
                muted
                playsInline
              >
                <source src={videoUrl} type="video/webm" />
              </video>
            ) : (
              children
            )}
          </Grid>
        </Grid>
      </Grid>
    );
  } else {
    return (
      <Grid container>
        <Grid
          className={clsx(classes.cushionedVert, classes.cusionedHorz)}
          item
          xs={12}
          container
          justify="center"
        >
          <Grid item>
            <Message />
          </Grid>
        </Grid>
        <Grid
          className={c2.fixLineHeight}
          item
          xs={12}
          container
          justify="center"
        >
          <Grid item>
            {!!videoUrl ? (
              <video autoPlay loop muted playsInline>
                <source src={videoUrl} type="video/webm" />
              </video>
            ) : (
              children
            )}
          </Grid>
        </Grid>
      </Grid>
    );
  }
}

function Example() {
  const classes = useStyles();

  return (
    <>
      <Value
        title="Report Messages"
        message="Users discreetly notify moderation about potentially rule-breaking messages."
        videoUrl={reportMessageWebm}
        spacingBelow
      />
      <Value
        title="Receive Reports"
        message="Incoming reports made by users get sent to staff members, where they can be handled."
        videoUrl={incomingReportWebm}
        spacingBelow
        isReversed
      />
      <Value
        title="Technically Awesome"
        message="Narc is built with Rust and open source on GitHub. That's how you know it's awesome."
      >
        <a href="https://github.com/SirJosh3917/narc">
          <img
            className={classes.openSource}
            src="https://opengraph.githubassets.com/c113fadd3293245835e5d88758ba26ec4d94e4737f16fd597dab3aafa36da612/SirJosh3917/narc"
            alt="Narc on Github"
          />
        </a>
      </Value>
      {/* TODO: don't blatantly copy HeroHeader */}

      <Grid
        className={clsx(
          classes.section,
          classes.centerText,
          classes.sectionLeftRightPadding
        )}
        container
        justify="center"
        direction="column"
      >
        <Grid item container justify="center">
          <Typography
            className={classes.stupidHack}
            variant="h3"
            component="h2"
          >
            Ready?
          </Typography>
        </Grid>
        <Grid
          item
          container
          justify="center"
          alignItems="center"
          direction="row"
        >
          <Grid item>
            <Button
              className={classes.button}
              variant="contained"
              color="secondary"
              href={INVITE_DISCORD_URL}
            >
              <span className={classes.buttonText}>Invite Now</span>
            </Button>
          </Grid>
        </Grid>
      </Grid>
    </>
  );
}

export default function LandingPage() {
  return (
    <>
      <HeroHeader />
      <Example />
    </>
  );
}
