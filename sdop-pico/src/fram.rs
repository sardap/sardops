use embassy_rp::gpio::Output;
use embedded_hal::spi::{SpiBus, SpiDevice};

pub const SDOP_SAVE_ADDR: u16 = 0x100;

const CMD_WREN: u8 = 0x06;
const CMD_WRITE: u8 = 0x02;
const CMD_READ: u8 = 0x03;

pub async fn write_enable<SPI: SpiBus>(
    spi: &mut SPI,
    cs: &mut Output<'_>,
) -> Result<(), SPI::Error> {
    cs.set_low();
    spi.write(&[CMD_WREN])?;
    cs.set_high();
    Ok(())
}

pub async fn write<SPI: SpiBus>(
    spi: &mut SPI,
    cs: &mut Output<'_>,
    addr: u16,
    data: &[u8],
) -> Result<(), SPI::Error> {
    if let Err(err) = write_enable(spi, cs).await {
        return Err(err);
    }
    let tx_prefix = [CMD_WRITE, (addr >> 8) as u8, (addr & 0xFF) as u8];
    cs.set_low();
    spi.write(&tx_prefix)?;
    spi.write(data)?;
    cs.set_high();
    Ok(())
}

pub async fn read<SPI: SpiBus>(
    spi: &mut SPI,
    cs: &mut Output<'_>,
    addr: u16,
    buf: &mut [u8],
) -> Result<(), SPI::Error> {
    let tx_prefix = [CMD_READ, (addr >> 8) as u8, (addr & 0xFF) as u8];
    cs.set_low();
    spi.write(&tx_prefix);
    spi.read(buf);
    cs.set_high();
    Ok(())
}
