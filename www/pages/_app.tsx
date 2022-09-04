import "../styles/globals.css";
import type { AppProps } from "next/app";

import AppBar from "@mui/material/AppBar";
import Paper from "@mui/material/Paper";
import Container from "@mui/material/Container";
import Toolbar from "@mui/material/Toolbar";
import Typography from "@mui/material/Typography";
import { createTheme, ThemeProvider } from "@mui/material/styles";
import CssBaseline from "@mui/material/CssBaseline";
import Box from "@mui/material/Box";
import Button from "@mui/material/Button";
import Link from "next/link";

const theme = createTheme({
  palette: {
    primary: {
      main: "#d71345",
    },
  },
});

function Header() {
  return (
    <AppBar position="static">
      <Toolbar>
        <Typography
          component="div"
          variant="h6"
          sx={{ mx: 2, fontWeight: "bold" }}
        >
          めん処{" "}
          <ruby>
            譽紫<rp>(</rp>
            <rt>よし</rt>
            <rp>)</rp>
          </ruby>
        </Typography>

        <Box sx={{ flexGrow: 1, display: { md: "flex" } }}>
          <Link href="/" passHref>
            <Button sx={{ my: 2, color: "white", display: "block" }}>
              実行履歴
            </Button>
          </Link>

          <Link href="/problems" passHref>
            <Button sx={{ my: 2, color: "white", display: "block" }}>
              問題一覧
            </Button>
          </Link>

          <Link href="/submit" passHref>
            <Button sx={{ my: 2, color: "white", display: "block" }}>
              ソルバー実行
            </Button>
          </Link>
        </Box>
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
