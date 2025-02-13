import { useState } from "react";
import reactLogo from "./assets/react.svg";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";
import Button from '@mui/material/Button';
import TextField from '@mui/material/TextField';
import Checkbox from '@mui/material/Checkbox';
import FormControlLabel from '@mui/material/FormControlLabel';
import Switch from '@mui/material/Switch'
import DesignServicesOutlinedIcon from '@mui/icons-material/DesignServicesOutlined';
import ChromeReaderModeOutlinedIcon from '@mui/icons-material/ChromeReaderModeOutlined';

async function getProcessId(processName) {
  return await invoke('get_process_id', { processName });
}

async function readMemory(processId, address, size) {
  return await invoke('read_memory', { processId, address, size });
}

async function writeMemory(processId, address, data) {
  return await invoke('write_memory', { processId, address, data });
}

async function readMemoryWithOffsets(processId, baseAddress, offsets) {
  return await invoke('read_memory_with_offsets', { processId, baseAddress, offsets, size: 4 });
}

async function writeMemoryWithOffsets(processId, baseAddress, offsets, data) {
  return await invoke('write_memory_with_offsets', { processId, baseAddress, offsets, data });
}
async function getModuleBaseAddress(processId, moduleName) {
  return await invoke('get_module_base_address', { pid: processId, moduleName });
}

async function allocateMemory(processId, size) {
  try {
    const address = await invoke("allocate_memory_command", { processId: processId, size: size });
    return address;
  } catch (error) {
    throw error;
  }
}

function bytesToIntLittleEndian(bytes) {
  let value = 0;
  for (let i = 0; i < bytes.length; i++) {
    value += bytes[i] << (8 * i);
  }
  return value;
}

function string2hex(input) {
  const numbers = input.split(' ');
  const hexIntArray = numbers.map(num => parseInt(num, 16));
  return hexIntArray;
}

function createJumpInstruction(targetAddress, currentAddress) {
  const nextInstructionAddress = currentAddress + 5;
  const offset = targetAddress - nextInstructionAddress;
  const offsetBytes = new Uint8Array(4);
  new DataView(offsetBytes.buffer).setUint32(0, offset, true);
  const jumpInstruction = new Uint8Array(5);
  jumpInstruction[0] = 0xE9;
  jumpInstruction.set(offsetBytes, 1);

  const hexString = Array.from(jumpInstruction)
    .map(byte => byte.toString(16).padStart(2, '0').toUpperCase())
    .join(' ');

  return hexString;
}



function App() {
  const processName = "PlantsVsZombies.exe";

  const [sunshineValue, setSunshineValue] = useState(0);
  async function sunshineModify(isRead) {
    const processId = await getProcessId(processName)
    const baseAddress = 0x006A9EC0;
    const goldValue = parseInt(sunshineValue);
    const offsets = [0x768, 0x5560]
    const data = new Uint8Array(4);
    const dataView = new DataView(data.buffer);
    dataView.setUint32(0, goldValue, true);

    if (isRead) {
      const gold = await readMemoryWithOffsets(processId, baseAddress, offsets)
      setSunshineValue(bytesToIntLittleEndian(gold));
    } else {
      await writeMemoryWithOffsets(processId, baseAddress, offsets, Array.from(data));
    }
  }

  const [modeIdValue, setModeIdValue] = useState(0)

  async function modeModify(isRead) {
    const processId = await getProcessId(processName)
    const baseAddress = 0x006A9EC0;
    const goldValue = parseInt(modeIdValue);
    const offsets = [0x7F8]
    const data = new Uint8Array(4);
    const dataView = new DataView(data.buffer);
    dataView.setUint32(0, goldValue, true);
    if (isRead) {
      const modeId = await readMemoryWithOffsets(processId, baseAddress, offsets)
      setModeIdValue(bytesToIntLittleEndian(modeId));
    } else {

      await writeMemoryWithOffsets(processId, baseAddress, offsets, Array.from(data));
    }
  }

  async function sunshineInfinite(modify) {
    const processId = await getProcessId(processName)
    const targetAddress = 0x008AF806;
    var byteArray;
    if (modify) {
      byteArray = "90 90 90 90 90 90";
    } else {
      byteArray = "89 B7 60 55 00 00";
    }
    await writeMemory(processId, targetAddress, string2hex(byteArray));
  }

  async function noCooling(modify) {
    const processId = await getProcessId(processName)
    const targetAddress = 0x0048728C;
    var byteArray;
    if (modify) {
      byteArray = "81 47 24 99 09 00 00";
    } else {
      byteArray = "83 47 24 01 8B 47 24"
    }
    await writeMemory(processId, targetAddress, string2hex(byteArray));
  }

  async function columPlant(modify) {
    const processId = await getProcessId(processName)
    const targetAddress = 0x00410AE6;
    var byteArray;
    if (modify) {
      byteArray = "90 90 90 90 90 90";
    } else {
      byteArray = "0F 85 E5 00 00 00";
    }
    await writeMemory(processId, targetAddress, string2hex(byteArray));
  }

  async function overPlant(modify) {
    const processId = await getProcessId(processName)
    const firstAddress = await allocateMemory(processId, 2048);;
    const secondAddress = 0x0040E020;
    var secondByteArray;
    if (modify) {
      const firstByteArray = "31 C0 C2 0C 00 83 EC 18 53 55 E9 16 E0 F0 FE";
      secondByteArray = createJumpInstruction(firstAddress, secondAddress)
      await writeMemory(processId, firstAddress, string2hex(firstByteArray));
    } else {
      secondByteArray = "83 EC 18 53 55";
    }
    await writeMemory(processId, secondAddress, string2hex(secondByteArray));
  }

  return (
    <div style={{ width: "100%", height: "90vh", display: 'flex', flexDirection: "column", justifyContent: "center", alignItems: "center" }}>
      <div style={{ display: "flex", flexDirection: "row" }}>
        <div style={{ display: "flex", flexDirection: "column", maxWidth: 250 }} >
          <TextField onChange={(event) => { setSunshineValue(event.target.value) }} value={sunshineValue} type="number" id="filled-basic" label="阳光数量" variant="filled" />
          <div style={{ display: "flex" }}>
            <Button startIcon={<DesignServicesOutlinedIcon />} style={{ marginTop: "10px", flex: 1 }} variant="contained" onClick={
              () => {
                sunshineModify(false)
              }
            }>修改</Button>
            <div style={{ width: "8px" }}></div>
            <Button startIcon={<ChromeReaderModeOutlinedIcon />} color="success" style={{ marginTop: "10px" }} variant="contained" onClick={
              () => {
                sunshineModify(true)
              }
            }>读取</Button>
          </div>
        </div>
        <div style={{ width: "8px" }}></div>
        <div style={{ display: "flex", flexDirection: "column", maxWidth: 250 }} >
          <TextField onChange={(event) => { setModeIdValue(event.target.value) }} value={modeIdValue} type="number" id="filled-basic" label="模式ID" variant="filled" />
          <div style={{ display: "flex" }}>
            <Button startIcon={<DesignServicesOutlinedIcon />} style={{ marginTop: "10px", flex: 1 }} variant="contained" onClick={
              () => {
                modeModify(false)
              }
            }>修改</Button>
            <div style={{ width: "8px" }}></div>
            <Button startIcon={<ChromeReaderModeOutlinedIcon />} color="success" style={{ marginTop: "10px", flex: 1 }} variant="contained" onClick={
              () => {
                modeModify(true)
              }
            }>读取</Button>
          </div>
        </div>
      </div>
      <div>
        <FormControlLabel control={
          <Checkbox
            onChange={(event) => {
              sunshineInfinite(event.target.checked)
            }
            } />
        } label="阳光不减" />
        <FormControlLabel control={
          <Checkbox
            onChange={(event) => {
              noCooling(event.target.checked)
            }
            } />
        } label="无冷却" />
        <FormControlLabel control={
          <Checkbox
            onChange={(event) => {
              columPlant(event.target.checked)
            }
            } />
        } label="竖排种植" />
        <FormControlLabel control={
          <Checkbox
            onChange={(event) => {
              overPlant(event.target.checked)
            }
            } />
        } label="重叠种植" />
      </div>

    </div>
  );
}

export default App;
