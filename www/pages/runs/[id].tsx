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

import { fetchRun, RunResult } from "../../lib/db";
import React from "react";

interface Props {
  result: RunResult;
}

export const getServerSideProps: GetServerSideProps<Props> = async (
  context
) => {
  const { id } = context.query;
  if (typeof id !== "string") {
    throw Error(`invalid path parameter: ${JSON.stringify(id)}`);
  }

  const result = await fetchRun(id);
  return {
    props: { result },
  };
};

const Page: NextPage<Props> = ({ result }) => {
  return (
    <Stack spacing={2}>
      <Typography component="h1" variant="h5">
        {new Date(1000 * result.time).toLocaleString()}
      </Typography>

      <ul>
        <li>スコア: {result.score}</li>
        <li>回答数: {result.results.length}</li>
        <li>
          引数: <code>{result.args}</code>
        </li>
      </ul>

      <TableContainer>
        <Table>
          <TableHead>
            <TableRow>
              <TableCell>問題</TableCell>
              <TableCell>スコア</TableCell>
              <TableCell>入力</TableCell>
              <TableCell>出力</TableCell>
              <TableCell>ISL</TableCell>
              <TableCell>コミット</TableCell>
            </TableRow>
          </TableHead>
          <TableBody>
            {result.results.map((r) => {
              const url = `https://github.com/seikichi/icfpc2022/commit/${r.commit}`;
              const input = `https://cdn.robovinci.xyz/imageframes/${r.problemId}.png`;
              const output = `https://d30a5x02adw8tj.cloudfront.net/${result.id}/${r.problemId}.png`;
              const isl = `https://d30a5x02adw8tj.cloudfront.net/${result.id}/${r.problemId}.isl`;
              const initial = `https://cdn.robovinci.xyz/imageframes/${r.problemId}.initial.png`;
              return (
                <TableRow key={r.problemId}>
                  <TableCell>
                    <Link href={`/problems/${r.problemId}`} passHref>
                      <MuiLink>{r.problemId}</MuiLink>
                    </Link>
                  </TableCell>
                  <TableCell>{r.score}</TableCell>
                  <TableCell>
                    {r.problemId > 25 && (
                      <img
                        style={{
                          border: "1px solid black",
                          marginRight: "5px",
                        }}
                        src={initial}
                        width="80"
                        alt="output"
                      />
                    )}
                    <img
                      style={{ border: "1px solid black" }}
                      src={input}
                      width="80"
                      alt="input"
                    />
                  </TableCell>
                  <TableCell>
                    <img
                      style={{ border: "1px solid black" }}
                      src={output}
                      width="80"
                      alt="output"
                    />
                  </TableCell>
                  <TableCell>
                    <Link href={isl} passHref>
                      <MuiLink target="_blank">{r.problemId}.isl</MuiLink>
                    </Link>
                  </TableCell>
                  <TableCell>
                    <Link href={url} passHref>
                      <MuiLink target="_blank">{r.commit}</MuiLink>
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
