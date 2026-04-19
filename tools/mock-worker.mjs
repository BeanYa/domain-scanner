import http from "node:http";

const port = Number(process.env.PORT || 8731);
const workerId = process.env.WORKER_ID || "mock-worker";
const workerName = process.env.WORKER_NAME || "Mock Worker";
const token = process.env.WORKER_TOKEN || "token-dev";
const batches = new Map();

function json(res, status, payload) {
  res.writeHead(status, { "content-type": "application/json" });
  res.end(JSON.stringify(payload));
}

function unauthorized(res) {
  json(res, 401, { error: { code: "invalid_token", message: "Invalid worker token" } });
}

function readBody(req) {
  return new Promise((resolve, reject) => {
    let body = "";
    req.on("data", (chunk) => {
      body += chunk;
    });
    req.on("end", () => {
      try {
        resolve(body ? JSON.parse(body) : {});
      } catch (error) {
        reject(error);
      }
    });
  });
}

function checkAuth(req, res) {
  const header = req.headers.authorization || "";
  if (header !== `Bearer ${token}`) {
    unauthorized(res);
    return false;
  }
  return true;
}

function capabilities() {
  return {
    max_running_batches: 4,
    max_total_concurrency: 200,
    max_batch_concurrency: 100,
    supports_pause: true,
    supports_cancel: true,
    supports_proxy_config: true,
  };
}

function makeResult(batch, index) {
  const tld = batch.tlds[index % batch.tlds.length] || ".com";
  const itemIndex = batch.start_index + index;
  return {
    seq: index + 1,
    item_index: itemIndex,
    domain: `mock-${itemIndex}${tld}`,
    tld,
    status: index % 5 === 0 ? "unavailable" : "available",
    is_available: index % 5 !== 0,
    query_method: "mock",
    response_time_ms: 10 + (index % 20),
    error_message: null,
    checked_at: new Date().toISOString(),
  };
}

function ensureBatch(body) {
  const existing = batches.get(body.batch_id);
  if (existing) return existing;

  const requestCount = Math.max(0, Number(body.end_index) - Number(body.start_index));
  const items = Array.from({ length: requestCount }, (_, index) => makeResult(body, index));
  const logs = [
    {
      seq: 1,
      level: "info",
      log_type: "task",
      message: `Accepted batch ${body.batch_id}`,
      created_at: new Date().toISOString(),
    },
  ];
  const batch = {
    ...body,
    status: "succeeded",
    attempt: body.attempt || 0,
    request_count: requestCount,
    completed_count: requestCount,
    available_count: items.filter((item) => item.status === "available").length,
    error_count: 0,
    result_cursor: items.length,
    log_cursor: logs.length,
    started_at: new Date().toISOString(),
    updated_at: new Date().toISOString(),
    finished_at: new Date().toISOString(),
    error_message: null,
    items,
    logs,
  };
  batches.set(body.batch_id, batch);
  return batch;
}

function pageItems(items, afterSeq, limit) {
  const selected = items.filter((item) => item.seq > afterSeq).slice(0, limit);
  const nextSeq = selected.at(-1)?.seq ?? afterSeq;
  return {
    items: selected,
    next_seq: nextSeq,
    has_more: items.some((item) => item.seq > nextSeq),
  };
}

const server = http.createServer(async (req, res) => {
  const url = new URL(req.url || "/", `http://${req.headers.host}`);

  if (!checkAuth(req, res)) return;

  if (req.method === "GET" && url.pathname === "/health") {
    json(res, 200, {
      ok: true,
      worker_id: workerId,
      worker_name: workerName,
      version: "0.1.0-mock",
      status: "available",
      capabilities: capabilities(),
    });
    return;
  }

  if (req.method === "GET" && url.pathname === "/capabilities") {
    json(res, 200, capabilities());
    return;
  }

  if (req.method === "POST" && url.pathname === "/batches") {
    const body = await readBody(req);
    const batch = ensureBatch(body);
    json(res, 200, { accepted: true, batch_id: batch.batch_id, status: batch.status });
    return;
  }

  const statusMatch = url.pathname.match(/^\/batches\/([^/]+)\/status$/);
  if (req.method === "GET" && statusMatch) {
    const batch = batches.get(statusMatch[1]);
    if (!batch) {
      json(res, 404, { error: { code: "not_found", message: "Batch not found" } });
      return;
    }
    json(res, 200, {
      batch_id: batch.batch_id,
      status: batch.status,
      attempt: batch.attempt,
      request_count: batch.request_count,
      completed_count: batch.completed_count,
      available_count: batch.available_count,
      error_count: batch.error_count,
      result_cursor: batch.result_cursor,
      log_cursor: batch.log_cursor,
      started_at: batch.started_at,
      updated_at: batch.updated_at,
      finished_at: batch.finished_at,
      error_message: batch.error_message,
    });
    return;
  }

  const resultsMatch = url.pathname.match(/^\/batches\/([^/]+)\/results$/);
  if (req.method === "GET" && resultsMatch) {
    const batch = batches.get(resultsMatch[1]);
    if (!batch) {
      json(res, 404, { error: { code: "not_found", message: "Batch not found" } });
      return;
    }
    const page = pageItems(
      batch.items,
      Number(url.searchParams.get("after_seq") || 0),
      Number(url.searchParams.get("limit") || 500)
    );
    json(res, 200, { batch_id: batch.batch_id, ...page });
    return;
  }

  const logsMatch = url.pathname.match(/^\/batches\/([^/]+)\/logs$/);
  if (req.method === "GET" && logsMatch) {
    const batch = batches.get(logsMatch[1]);
    if (!batch) {
      json(res, 404, { error: { code: "not_found", message: "Batch not found" } });
      return;
    }
    const page = pageItems(
      batch.logs,
      Number(url.searchParams.get("after_seq") || 0),
      Number(url.searchParams.get("limit") || 500)
    );
    json(res, 200, { batch_id: batch.batch_id, ...page });
    return;
  }

  const controlMatch = url.pathname.match(/^\/batches\/([^/]+)\/(pause|cancel)$/);
  if (req.method === "POST" && controlMatch) {
    const batch = batches.get(controlMatch[1]);
    if (!batch) {
      json(res, 404, { error: { code: "not_found", message: "Batch not found" } });
      return;
    }
    batch.status = controlMatch[2] === "pause" ? "paused" : "cancelled";
    batch.updated_at = new Date().toISOString();
    json(res, 200, { ok: true });
    return;
  }

  json(res, 404, { error: { code: "not_found", message: "Route not found" } });
});

server.listen(port, () => {
  console.log(`Mock worker listening on http://127.0.0.1:${port}`);
  console.log(`Authorization: Bearer ${token}`);
});
