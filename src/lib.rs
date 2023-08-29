#![no_std]

use core::fmt;

pub use embedded_hal as hal;
use hal::digital::v2::PinState;

pub use generic_array;
use generic_array::{ArrayLength, GenericArray};

pub trait Same<T> {}

impl<T> Same<T> for T {}

pub trait ParallelBus {
    type BusWidth: ArrayLength<PinState>;
}

pub trait InputBus: ParallelBus {
    type Error: fmt::Debug;
    fn read_bus(&self) -> Result<GenericArray<PinState, Self::BusWidth>, Self::Error>;
}

pub trait OutputBus: ParallelBus {
    type Error: fmt::Debug;
    fn write_bus(
        &mut self,
        states: &GenericArray<PinState, Self::BusWidth>,
    ) -> Result<(), Self::Error>;
}

pub trait IoBus<TInput, TOutput>: ParallelBus
where
    TInput: InputBus + IoBus<TInput, TOutput>,
    TOutput: OutputBus + IoBus<TInput, TOutput>,
    <TInput as ParallelBus>::BusWidth: Same<<TOutput as ParallelBus>::BusWidth>,
    <TOutput as ParallelBus>::BusWidth: Same<<TInput as ParallelBus>::BusWidth>,
{
    type IntoInputError: fmt::Debug;
    type IntoOutputError: fmt::Debug;
    fn into_input_bus(self) -> Result<TInput, Self::IntoInputError>;
    fn into_output_bus(self) -> Result<TOutput, Self::IntoOutputError>;
}

pub trait BidirectionBus<TInput, TOutput>: IoBus<TInput, TOutput>
where
    TInput: InputBus + IoBus<TInput, TOutput>,
    TOutput: OutputBus + IoBus<TInput, TOutput>,
    <TInput as ParallelBus>::BusWidth: Same<<TOutput as ParallelBus>::BusWidth>,
    <TOutput as ParallelBus>::BusWidth: Same<<TInput as ParallelBus>::BusWidth>,
{
    fn switch_to_input_bus(&mut self) -> Result<&TInput, Self::IntoInputError>;
    fn switch_to_output_bus(&mut self) -> Result<&mut TOutput, Self::IntoOutputError>;
}

#[derive(Debug)]
pub enum DirectedBusEnum<TInput, TOutput>
where
    TInput: InputBus + IoBus<TInput, TOutput>,
    TOutput: OutputBus + IoBus<TInput, TOutput>,
    <TInput as ParallelBus>::BusWidth: Same<<TOutput as ParallelBus>::BusWidth>,
    <TOutput as ParallelBus>::BusWidth: Same<<TInput as ParallelBus>::BusWidth>,
{
    InputBus(TInput),
    OutputBus(TOutput),
}

impl<TInput, TOutput> ParallelBus for DirectedBusEnum<TInput, TOutput>
where
    TInput: InputBus + IoBus<TInput, TOutput>,
    TOutput: OutputBus + IoBus<TInput, TOutput>,
    <TInput as ParallelBus>::BusWidth: Same<<TOutput as ParallelBus>::BusWidth>,
    <TOutput as ParallelBus>::BusWidth: Same<<TInput as ParallelBus>::BusWidth>,
{
    type BusWidth = <TInput as ParallelBus>::BusWidth;
}

impl<TInput, TOutput> IoBus<TInput, TOutput> for DirectedBusEnum<TInput, TOutput>
where
    TInput: InputBus + IoBus<TInput, TOutput>,
    TOutput: OutputBus + IoBus<TInput, TOutput>,
    <TInput as ParallelBus>::BusWidth: Same<<TOutput as ParallelBus>::BusWidth>,
    <TOutput as ParallelBus>::BusWidth: Same<<TInput as ParallelBus>::BusWidth>,
{
    type IntoInputError = <TOutput as IoBus<TInput, TOutput>>::IntoInputError;
    type IntoOutputError = <TInput as IoBus<TInput, TOutput>>::IntoOutputError;

    fn into_input_bus(self) -> Result<TInput, Self::IntoInputError> {
        match self {
            Self::InputBus(bus) => Ok(bus),
            Self::OutputBus(bus) => bus.into_input_bus(),
        }
    }

    fn into_output_bus(self) -> Result<TOutput, Self::IntoOutputError> {
        match self {
            Self::InputBus(bus) => bus.into_output_bus(),
            Self::OutputBus(bus) => Ok(bus),
        }
    }
}

#[derive(Debug)]
pub struct DirectedBus<TInput, TOutput>(Option<DirectedBusEnum<TInput, TOutput>>)
where
    TInput: InputBus + IoBus<TInput, TOutput>,
    TOutput: OutputBus + IoBus<TInput, TOutput>,
    <TInput as ParallelBus>::BusWidth: Same<<TOutput as ParallelBus>::BusWidth>,
    <TOutput as ParallelBus>::BusWidth: Same<<TInput as ParallelBus>::BusWidth>;

impl<TInput, TOutput> From<DirectedBusEnum<TInput, TOutput>> for DirectedBus<TInput, TOutput>
where
    TInput: InputBus + IoBus<TInput, TOutput>,
    TOutput: OutputBus + IoBus<TInput, TOutput>,
    <TInput as ParallelBus>::BusWidth: Same<<TOutput as ParallelBus>::BusWidth>,
    <TOutput as ParallelBus>::BusWidth: Same<<TInput as ParallelBus>::BusWidth>,
{
    #[inline]
    fn from(value: DirectedBusEnum<TInput, TOutput>) -> Self {
        Self(Some(value))
    }
}

impl<TInput, TOutput> From<DirectedBus<TInput, TOutput>> for DirectedBusEnum<TInput, TOutput>
where
    TInput: InputBus + IoBus<TInput, TOutput>,
    TOutput: OutputBus + IoBus<TInput, TOutput>,
    <TInput as ParallelBus>::BusWidth: Same<<TOutput as ParallelBus>::BusWidth>,
    <TOutput as ParallelBus>::BusWidth: Same<<TInput as ParallelBus>::BusWidth>,
{
    #[inline]
    fn from(value: DirectedBus<TInput, TOutput>) -> Self {
        value.0.unwrap()
    }
}

impl<TInput, TOutput> ParallelBus for DirectedBus<TInput, TOutput>
where
    TInput: InputBus + IoBus<TInput, TOutput>,
    TOutput: OutputBus + IoBus<TInput, TOutput>,
    <TInput as ParallelBus>::BusWidth: Same<<TOutput as ParallelBus>::BusWidth>,
    <TOutput as ParallelBus>::BusWidth: Same<<TInput as ParallelBus>::BusWidth>,
{
    type BusWidth = <DirectedBusEnum<TInput, TOutput> as ParallelBus>::BusWidth;
}

impl<TInput, TOutput> IoBus<TInput, TOutput> for DirectedBus<TInput, TOutput>
where
    TInput: InputBus + IoBus<TInput, TOutput>,
    TOutput: OutputBus + IoBus<TInput, TOutput>,
    <TInput as ParallelBus>::BusWidth: Same<<TOutput as ParallelBus>::BusWidth>,
    <TOutput as ParallelBus>::BusWidth: Same<<TInput as ParallelBus>::BusWidth>,
{
    type IntoInputError = <TOutput as IoBus<TInput, TOutput>>::IntoInputError;
    type IntoOutputError = <TInput as IoBus<TInput, TOutput>>::IntoOutputError;

    #[inline]
    fn into_input_bus(self) -> Result<TInput, Self::IntoInputError> {
        self.0.unwrap().into_input_bus()
    }

    #[inline]
    fn into_output_bus(self) -> Result<TOutput, Self::IntoOutputError> {
        self.0.unwrap().into_output_bus()
    }
}

impl<TInput, TOutput> BidirectionBus<TInput, TOutput> for DirectedBus<TInput, TOutput>
where
    TInput: InputBus + IoBus<TInput, TOutput>,
    TOutput: OutputBus + IoBus<TInput, TOutput>,
    <TInput as ParallelBus>::BusWidth: Same<<TOutput as ParallelBus>::BusWidth>,
    <TOutput as ParallelBus>::BusWidth: Same<<TInput as ParallelBus>::BusWidth>,
{
    fn switch_to_input_bus(&mut self) -> Result<&TInput, Self::IntoInputError> {
        let bus = self.0.take().unwrap();
        self.0 = Some(DirectedBusEnum::InputBus(bus.into_input_bus()?));
        match self.0.as_ref().unwrap() {
            DirectedBusEnum::InputBus(bus) => Ok(bus),
            _ => panic!(),
        }
    }

    fn switch_to_output_bus(&mut self) -> Result<&mut TOutput, Self::IntoOutputError> {
        let bus = self.0.take().unwrap();
        self.0 = Some(DirectedBusEnum::OutputBus(bus.into_output_bus()?));
        match self.0.as_mut().unwrap() {
            DirectedBusEnum::OutputBus(bus) => Ok(bus),
            _ => panic!(),
        }
    }
}
