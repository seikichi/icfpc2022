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

const NUM_PROBLEMS = 40;

const Page: NextPage = () => {
  const ids = Array.from(new Array(NUM_PROBLEMS), (_, i) => i);

  return (
    <Stack spacing={2}>
      <Typography component="h1" variant="h5">
        問題一覧
      </Typography>

      <TableContainer>
        <Table>
          <TableHead>
            <TableRow>
              <TableCell>#</TableCell>
              <TableCell>入力</TableCell>
            </TableRow>
          </TableHead>
          <TableBody>
            {ids.map((i) => {
              const problemId = i + 1;
              const url = `https://cdn.robovinci.xyz/imageframes/${problemId}.png`;
              const initial = `https://cdn.robovinci.xyz/imageframes/${problemId}.initial.png`;
              return (
                <TableRow key={i}>
                  <TableCell>
                    <Link href={`/problems/${problemId}`} passHref>
                      <MuiLink>{problemId}</MuiLink>
                    </Link>
                  </TableCell>
                  <TableCell>
                    {problemId > 25 && (
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
                      src={url}
                      width="80"
                      alt="output"
                    />
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
