import { maxHeaderSize } from "http";
import type { NextApiRequest, NextApiResponse } from "next";
import { v4 as uuidv4 } from "uuid";
import { fetchRunList, putRun } from "../../lib/db";
import { schema } from "../../lib/schema/run";

export type SubmitResult =
  | { success: true }
  | { success: false; message: string };

export default async function handler(
  req: NextApiRequest,
  res: NextApiResponse<SubmitResult>
) {
  if (req.method !== "POST") {
    return res
      .status(405)
      .json({ success: false, message: "未サポートのメソッドです" });
  }

  const body = schema.safeParse(req.body);
  if (!body.success) {
    return res
      .status(400)
      .json({ success: false, message: "不正なパラメーターです" });
  }

  // 時刻チェック
  const prevs = await fetchRunList();
  const last = Math.max(0, ...prevs.map((p) => p.time));
  const current = new Date().getTime() / 1000;
  if (last > current - 60 * 5) {
    return res
      .status(400)
      .json({ success: false, message: "実行は5分に1回許可されています" });
  }

  // DB 追加したり Lambda 実行したりする
  const { args, target } = body.data;
  const uuid = uuidv4();

  try {
    await putRun({ uuid, args, target });
  } catch (e) {
    console.error(e);
    res.status(200).json({ success: false, message: "登録に失敗しました" });
    return;
  }

  res.status(200).json({ success: true });
}
