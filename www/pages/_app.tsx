import "../styles/globals.css";
import type { AppProps } from "next/app";

import AppBar from "@mui/material/AppBar";
import Paper from "@mui/material/Paper";
import Container from "@mui/material/Container";
import Toolbar from "@mui/material/Toolbar";
import Typography from "@mui/material/Typography";
import { createTheme, ThemeProvider } from "@mui/material/styles";
import Link from "next/link";
import CssBaseline from "@mui/material/CssBaseline";

const theme = createTheme({});

function Header() {
  // NOTE: cursor: "pointer" が無いとマウスカーソルが変わらない...
  return (
    <AppBar position="static">
      <Toolbar>
        <Link href="/" passHref>
          <Typography
            component="div"
            variant="h6"
            sx={{ flexGrow: 1, cursor: "pointer" }}
          >
            めん処譽紫
          </Typography>
        </Link>
      </Toolbar>
    </AppBar>
  );
}

function MyApp({ Component, pageProps: { session, ...pageProps } }: AppProps) {
  return (
    <ThemeProvider theme={theme}>
      <CssBaseline />
      <Header />
      <Container component="main" maxWidth="xl">
        <Paper
          variant="outlined"
          sx={{ my: 4, px: 8, py: 2, borderRadius: "16px" }}
        >
          <Component {...pageProps} />
        </Paper>
      </Container>
    </ThemeProvider>
  );
}

export default MyApp;
