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

import { useEffect } from "react";

import { fetchRun, RunResult } from "../../lib/db";

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
  console.log(result);
  return (
    <>
      概要:
      <ul>
        <li>ID: {result.id}</li>
        <li>Score: {result.score}</li>
        <li>Date: {new Date(1000 * result.time).toLocaleString()}</li>
        <li>AI: {result.ai}</li>
      </ul>
      各タスク:
      <ul>
        {result.results.map((r) => {
          return (
            <li key={r.problemId}>
              Problem: {r.problemId}, Score: {r.score}
            </li>
          );
        })}
      </ul>
    </>
  );
};

export default Page;
