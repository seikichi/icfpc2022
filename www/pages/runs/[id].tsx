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

import { fetchRun } from "../../lib/db";

interface Props {
  result: any;
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
  return <>Hello, world!</>;
};

export default Page;
