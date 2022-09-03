/* eslint-disable @next/next/no-img-element */
import type { NextPage, GetServerSideProps } from "next";
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

import { fetchRun, fetchSolutionList, RunResult, Solution } from "../../lib/db";
import React from "react";

interface Props {
  id: string;
  solutions: Solution[];
}

export const getServerSideProps: GetServerSideProps<Props> = async (
  context
) => {
  const { id } = context.query;
  if (typeof id !== "string") {
    throw Error(`invalid path parameter: ${JSON.stringify(id)}`);
  }
  const solutions = await fetchSolutionList(id);

  return {
    props: { id, solutions },
  };
};

const Page: NextPage<Props> = ({ id, solutions }) => {
  const input = `https://cdn.robovinci.xyz/imageframes/${id}.png`;

  return (
    <Stack spacing={2}>
      <Typography component="h1" variant="h5">
        Problem {id}
      </Typography>

      <img src={input} width="200" alt="problem" />

      <TableContainer>
        <Table>
          <TableHead>
            <TableRow>
              <TableCell>ランク</TableCell>
              <TableCell>スコア</TableCell>
              <TableCell>AI</TableCell>
              <TableCell>実行ID</TableCell>
              <TableCell>出力</TableCell>
              <TableCell>コミット</TableCell>
            </TableRow>
          </TableHead>
          <TableBody>
            {solutions.map((s, i) => {
              const url = `https://github.com/seikichi/icfpc2022/commit/${s.commit}`;
              const output = `https://d30a5x02adw8tj.cloudfront.net/${s.runId}/${s.problemId}.png`;
              return (
                <TableRow key={i}>
                  <TableCell>{i + 1}</TableCell>
                  <TableCell>{s.score}</TableCell>
                  <TableCell>{s.ai}</TableCell>
                  <TableCell>
                    <Link href={`/runs/${s.runId}`} passHref>
                      <MuiLink>{s.runId}</MuiLink>
                    </Link>
                  </TableCell>
                  <TableCell>
                    <img src={output} width="80" alt="output" />
                  </TableCell>
                  <TableCell>
                    <Link href={url} passHref>
                      <MuiLink target="_blank">{s.commit}</MuiLink>
                    </Link>
                  </TableCell>
                </TableRow>
              );
            })}
          </TableBody>
        </Table>
      </TableContainer>
    </Stack>
  );
};

export default Page;
