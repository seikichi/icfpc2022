import type { NextPage } from "next";
import Link from "next/link";

import Stack from "@mui/material/Stack";
import Typography from "@mui/material/Typography";
import Table from "@mui/material/Table";
import TableBody from "@mui/material/TableBody";
import TableCell from "@mui/material/TableCell";
import TableContainer from "@mui/material/TableContainer";
import TableHead from "@mui/material/TableHead";
import TableRow from "@mui/material/TableRow";
import MuiLink from "@mui/material/Link";

import { fetchRunList, Run } from "../lib/db";

export async function getServerSideProps() {
  const runs = await fetchRunList();
  return {
    props: { runs },
  };
}

interface Props {
  runs: Run[];
}

const Home: NextPage<Props> = ({ runs }) => {
  console.log(runs);
  return (
    <Stack spacing={2}>
      <Typography component="h1" variant="h5">
        実行履歴
      </Typography>

      <TableContainer>
        <Table>
          <TableHead>
            <TableRow>
              <TableCell>実行日時</TableCell>
              <TableCell>引数</TableCell>
              <TableCell>問題数</TableCell>
              <TableCell>スコア</TableCell>
            </TableRow>
          </TableHead>
          <TableBody>
            {runs.map((r) => {
              const date = new Date(1000 * r.time).toLocaleString();
              return (
                <TableRow key={r.id}>
                  <TableCell>
                    <Link href={`/runs/${r.id}`} passHref>
                      <MuiLink>{date}</MuiLink>
                    </Link>
                  </TableCell>
                  <TableCell>{r.args}</TableCell>
                  <TableCell>{r.problems}</TableCell>
                  <TableCell>{r.score}</TableCell>
                </TableRow>
              );
            })}
          </TableBody>
        </Table>
      </TableContainer>
    </Stack>
  );
};

export default Home;
