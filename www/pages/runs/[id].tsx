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
import Breadcrumbs from "@mui/material/Breadcrumbs";
import Box from "@mui/material/Box";
import Tab from "@mui/material/Tab";
import Tabs from "@mui/material/Tabs";

import { fetchRun, RunResult } from "../../lib/db";
import React, { useCallback } from "react";

// 以下 https://mui.com/material-ui/react-tabs/ のパクリ
interface TabPanelProps {
  children?: React.ReactNode;
  index: number;
  value: number;
}

function TabPanel(props: TabPanelProps) {
  const { children, value, index, ...other } = props;

  return (
    <div
      role="tabpanel"
      hidden={value !== index}
      id={`simple-tabpanel-${index}`}
      aria-labelledby={`simple-tab-${index}`}
      {...other}
    >
      {value === index && <Box>{children}</Box>}
    </div>
  );
}

function a11yProps(index: number) {
  return {
    id: `simple-tab-${index}`,
    "aria-controls": `simple-tabpanel-${index}`,
  };
}

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
  const [value, setValue] = React.useState(0);

  const handleChange = useCallback(
    (_event: React.SyntheticEvent, newValue: number) => {
      setValue(newValue);
    },
    []
  );

  console.log(result.failures);
  return (
    <>
      <Breadcrumbs aria-label="breadcrumb">
        <Link href="/" passHref>
          <MuiLink underline="hover" color="inherit">
            実行履歴
          </MuiLink>
        </Link>
        <Typography color="text.primary">{result.id}</Typography>
      </Breadcrumbs>

      <Stack spacing={2} sx={{ my: 2 }}>
        <Typography component="h1" variant="h5">
          {new Date(1000 * result.time).toLocaleString()}
        </Typography>

        <ul>
          <li>スコア: {result.score}</li>
          <li>対象: {result.target}</li>
          <li>成功: {result.results.length} 件</li>
          <li>失敗: {result.failures.length} 件 (タイムアウト除く)</li>
          <li>
            引数: <code>{result.args}</code>
          </li>
        </ul>

        <Box sx={{ borderBottom: 1, borderColor: "divider" }}>
          <Tabs
            value={value}
            onChange={handleChange}
            aria-label="basic tabs example"
          >
            <Tab label="成功" {...a11yProps(0)} />
            <Tab label="失敗" {...a11yProps(1)} />
          </Tabs>
        </Box>
        <TabPanel value={value} index={0}>
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
                  <TableCell>時間(秒)</TableCell>
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
                      <TableCell>{r.time ?? "-"}</TableCell>
                    </TableRow>
                  );
                })}
              </TableBody>
            </Table>
          </TableContainer>
        </TabPanel>
        <TabPanel value={value} index={1}>
          <TableContainer>
            <Table>
              <TableHead>
                <TableRow>
                  <TableCell>問題</TableCell>
                  <TableCell>入力</TableCell>
                  <TableCell>エラー</TableCell>
                </TableRow>
              </TableHead>
              <TableBody>
                {result.failures.map((f) => {
                  const input = `https://cdn.robovinci.xyz/imageframes/${f.problemId}.png`;
                  const initial = `https://cdn.robovinci.xyz/imageframes/${f.problemId}.initial.png`;
                  return (
                    <TableRow key={f.problemId}>
                      <TableCell>
                        <Link href={`/problems/${f.problemId}`} passHref>
                          <MuiLink>{f.problemId}</MuiLink>
                        </Link>
                      </TableCell>
                      <TableCell>
                        {f.problemId > 25 && (
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
                      <TableCell>{f.error}</TableCell>
                    </TableRow>
                  );
                })}
              </TableBody>
            </Table>
          </TableContainer>
        </TabPanel>
      </Stack>
    </>
  );
};

export default Page;
