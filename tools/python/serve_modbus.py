import logging
import random
import threading

from pymodbus.server import StartTcpServer
from pymodbus.datastore import ModbusSequentialDataBlock
from pymodbus.datastore import ModbusSlaveContext, ModbusServerContext

logging.getLogger().setLevel(logging.INFO)

# Mimics the Sunspec Modbus Register Map
REGISTER_BLOCK_OFFSET = 40000
REGISTER_BLOCK_LENGTH = 69
REGISTER_BLOCK_DEFAULT = 255


class UpdatingDataBlock(ModbusSequentialDataBlock):
    # Always returns a random array
    def getValues(self, address, count=1):
        logging.info(f"event=getValues,address={address},count={count}")
        response = []
        for i in range(count):
            response.append(random.randint(0, REGISTER_BLOCK_DEFAULT))
        return response


def run_server():
    store = ModbusSlaveContext(
        ir=UpdatingDataBlock(REGISTER_BLOCK_OFFSET,
                             REGISTER_BLOCK_LENGTH * [REGISTER_BLOCK_DEFAULT]),
        zero_mode=True,
    )
    context = ModbusServerContext(slaves=store, single=True)
    logging.info("Starting pymodbus server")
    StartTcpServer(context=context, address=("0.0.0.0", 502))


if __name__ == "__main__":
    run_server()
