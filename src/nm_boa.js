// boa Native Messaging host
// Based on https://github.com/guest271314/NativeMessagingHosts/blob/main/nm_qjs_64.js
// guest271314 9-21-2026
const CHUNK_SIZE = 1024 * 1024; // 1MB absolute max chunk size
const COMMA = 44;
const OPEN_BRACKET = 91;
const CLOSE_BRACKET = 93;

async function readExactly(bytesToRead) {
  const buffer = new Uint8Array(bytesToRead);
  let totalRead = 0;
  while (totalRead < bytesToRead) {
    const view = buffer.subarray(totalRead);
    const n = std.read(view);
    if (n === null || n === 0) return null; 
    totalRead += n;
  }
  return buffer;
}

async function getMessage() {
  const lengthBytes = await readExactly(4);
  if (!lengthBytes) return null;

  const view = new DataView(lengthBytes.buffer);
  const messageLength = view.getUint32(0, true);

  return await readExactly(messageLength);
}

async function sendMessage(message) {
  if (message.length <= CHUNK_SIZE) {
    const len = message.length;
    const packet = new Uint8Array(4 + len);
    //const view = new DataView(packet.buffer);
    //view.setUint32(0, message.length, true);
    packet[0] = len & 255;
    packet[1] = len >> 8 & 255;
    packet[2] = len >> 16 & 255;
    packet[3] = len >> 24 & 255;
    packet.set(message, 4);
    
    let written = 0;
    while (written < packet.length) {
      const n = std.write(packet.subarray(written));
      if (n === 0) break;
      written += n;
    }
    return;
  }

  let index = 0;
  while (index < message.length) {
    let searchEnd = index + CHUNK_SIZE;
    if (searchEnd > message.length) searchEnd = message.length;

    let splitIndex = searchEnd;
    if (searchEnd < message.length) {
      let foundComma = -1;
      for (let i = searchEnd - 1; i >= index; i--) {
        if (message[i] === COMMA) {
          foundComma = i;
          break;
        }
      }
      splitIndex = foundComma !== -1 ? foundComma : searchEnd;
    }

    const rawChunk = message.subarray(index, splitIndex);
    if (rawChunk.length === 0) break;

    const startByte = rawChunk[0];
    const endByte = rawChunk[rawChunk.length - 1];

    let needsOpen = false;
    let needsClose = false;
    let body = rawChunk;

    if (startByte === OPEN_BRACKET) {
      if (endByte !== CLOSE_BRACKET) {
        needsClose = true;
      }
    } else if (startByte === COMMA) {
      needsOpen = true;
      body = rawChunk.subarray(1); 
      if (body[body.length - 1] !== CLOSE_BRACKET) {
        needsClose = true;
      }
    } else {
      needsOpen = true;
      needsClose = true;
    }

    const totalPayloadLen = (needsOpen ? 1 : 0) + body.length + (needsClose ? 1 : 0);
    const packet = new Uint8Array(4 + totalPayloadLen);
    packet[0] = totalPayloadLen & 255;
    packet[1] = totalPayloadLen >> 8 & 255;
    packet[2] = totalPayloadLen >> 16 & 255;
    packet[3] = totalPayloadLen >> 24 & 255;

    let offset = 4;
    if (needsOpen) {
      packet[offset] = OPEN_BRACKET;
      offset += 1;
    }
    packet.set(body, offset);
    offset += body.length;
    if (needsClose) {
      packet[offset] = CLOSE_BRACKET;
    }

    let written = 0;
    while (written < packet.length) {
      const n = std.write(packet.subarray(written));
      if (n === 0) break;
      written += n;
    }
    index = splitIndex;
  }
}

async function runHostLoop() {
  try {
    while (true) {
      const msg = await getMessage();
      if (!msg) break; 
      await sendMessage(msg);
    }
  } catch (e) {
    std.err(e.message);
  }
}

runHostLoop();
