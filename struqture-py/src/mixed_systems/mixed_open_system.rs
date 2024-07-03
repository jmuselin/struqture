// Copyright © 2021-2023 HQS Quantum Simulations GmbH. All Rights Reserved.
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except
// in compliance with the License. You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the
// License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either
// express or implied. See the License for the specific language governing permissions and
// limitations under the License.

use std::str::FromStr;

use super::{
    HermitianMixedProductWrapper, MixedDecoherenceProductWrapper, MixedHamiltonianSystemWrapper,
    MixedLindbladNoiseSystemWrapper,
};
use bincode::deserialize;
use pyo3::exceptions::{PyTypeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::PyByteArray;
use qoqo_calculator_pyo3::CalculatorComplexWrapper;
use struqture::mixed_systems::{
    HermitianMixedProduct, MixedDecoherenceProduct, MixedLindbladOpenSystem, OperateOnMixedSystems,
};
#[cfg(feature = "json_schema")]
use struqture::{MinSupportedVersion, STRUQTURE_VERSION};
use struqture::{OpenSystem, OperateOnDensityMatrix, StruqtureError};
use struqture_py_macros::noisy_system_wrapper;

/// These are representations of noisy systems of mixed_systems.
///
/// In a MixedLindbladOpenSystem is characterized by a MixedLindbladOpenOperator to represent the hamiltonian of the system, and an optional number of mixed_systems.
///
/// Args:
///     number_spins (List[Optional[int]]): The number of spin subsystems in the MixedLindbladOpenSystem.
///     number_bosons (List[Optional[int]]): The number of boson subsystems in the MixedLindbladOpenSystem.
///     number_fermions (List[Optional[int]]): The number of fermion subsystems in the MixedLindbladOpenSystem.
///
/// Returns:
///     self: The new MixedLindbladOpenSystem.
///
/// Examples
/// --------
///
/// .. code-block:: python
///     import numpy.testing as npt
///     import scipy.sparse as sp
///     from qoqo_calculator_pyo3 import CalculatorComplex, CalculatorFloat
///     from struqture_py.mixed_systems import MixedLindbladOpenSystem
///     from struqture_py.spins import DecoherenceProduct
///     from struqture_py.bosons import BosonProduct
///     from struqture_py.fermions import FermionProduct
///
///     slns = MixedLindbladOpenSystem()
///     dp = MixedDecoherenceProduct([DecoherenceProduct().z(0)], [BosonProduct([0], [1])], [FermionProduct([0], [0])])
///     slns.noise_add_operator_product((dp, dp), 2.0)
///     npt.assert_equal(slns.current_number_spins(), [1])
///     npt.assert_equal(slns.noise().get((dp, dp)), CalculatorFloat(2))
///
#[pyclass(
    name = "MixedLindbladOpenSystem",
    module = "struqture_py.mixed_systems"
)]
#[derive(Clone, Debug, PartialEq)]
pub struct MixedLindbladOpenSystemWrapper {
    /// Internal storage of [struqture::mixed_systems::MixedLindbladOpenSystem]
    pub internal: MixedLindbladOpenSystem,
}

#[noisy_system_wrapper(OpenSystem, OperateOnMixedSystems, Calculus)]
impl MixedLindbladOpenSystemWrapper {
    /// Create a new MixedLindbladOpenSystem.
    ///
    /// Args:
    ///     number_spins (List[Optional[int]]): The number of spin subsystems in the MixedLindbladOpenSystem.
    ///     number_bosons (List[Optional[int]]): The number of boson subsystems in the MixedLindbladOpenSystem.
    ///     number_fermions (List[Optional[int]]): The number of fermion subsystems in the MixedLindbladOpenSystem.
    ///
    /// Returns:
    ///     self: The new MixedLindbladOpenSystem.
    #[new]
    #[pyo3(signature = (
        number_spins = vec![None],
        number_bosons = vec![None],
        number_fermions = vec![None],
    ))]
    pub fn new(
        number_spins: Vec<Option<usize>>,
        number_bosons: Vec<Option<usize>>,
        number_fermions: Vec<Option<usize>>,
    ) -> Self {
        Self {
            internal: MixedLindbladOpenSystem::new(number_spins, number_bosons, number_fermions),
        }
    }

    /// Converts a struqture 2.x MixedLindbladOpenSystem to a struqture 2.x MixedLindbladOpenSystem.
    ///
    /// Args:
    ///     input (MixedLindbladOpenSystem): The struqture 2.x MixedLindbladOpenSystem to convert to struqture 1.x.
    ///
    /// Returns:
    ///     MixedLindbladOpenSystem: The struqture 1.x MixedLindbladOpenSystem created from the struqture 2.x MixedLindbladOpenSystem.
    ///
    /// Raises:
    ///     TypeError: If the input is not a struqture 2.x MixedLindbladOpenSystem.
    ///     ValueError: Conversion failed.
    #[staticmethod]
    pub fn from_struqture_two(input: &Bound<PyAny>) -> PyResult<MixedLindbladOpenSystemWrapper> {
        Python::with_gil(|_| -> PyResult<MixedLindbladOpenSystemWrapper> {
            let source_serialisation_meta = input.call_method0("_get_serialisation_meta").map_err(|_| {
                PyTypeError::new_err("Trying to use Python object as a struqture-py object that does not behave as struqture-py object. Are you sure you have the right type to all functions?".to_string())
            })?;
            let source_serialisation_meta: String = source_serialisation_meta.extract().map_err(|_| {
                PyTypeError::new_err("Trying to use Python object as a struqture-py object that does not behave as struqture-py object. Are you sure you have the right type to all functions?".to_string())
            })?;

            let source_serialisation_meta: struqture_two::StruqtureSerialisationMeta = serde_json::from_str(&source_serialisation_meta).map_err(|_| {
                PyTypeError::new_err("Trying to use Python object as a struqture-py object that does not behave as struqture-py object. Are you sure you have the right type to all functions?".to_string())
            })?;

            let target_serialisation_meta = <struqture_two::mixed_systems::MixedLindbladOpenSystem as struqture_two::SerializationSupport>::target_serialisation_meta();

            struqture_two::check_can_be_deserialised(
                &target_serialisation_meta,
                &source_serialisation_meta,
            )
            .map_err(|err| PyTypeError::new_err(err.to_string()))?;

            let get_bytes = input
                .call_method0("to_bincode")
                .map_err(|_| PyTypeError::new_err("Serialisation failed".to_string()))?;
            let bytes = get_bytes
                .extract::<Vec<u8>>()
                .map_err(|_| PyTypeError::new_err("Deserialisation failed".to_string()))?;
            let two_import: struqture_two::mixed_systems::MixedLindbladOpenSystem =
                deserialize(&bytes[..]).map_err(|err| {
                    PyTypeError::new_err(format!("Type conversion failed: {}", err))
                })?;
            let number_spins: usize = <struqture_two::mixed_systems::MixedLindbladOpenSystem as struqture_two::mixed_systems::OperateOnMixedSystems>::current_number_spins(&two_import).len();
            let spin_systems: Vec<Option<usize>> = vec![None; number_spins];
            let number_bosons: usize = <struqture_two::mixed_systems::MixedLindbladOpenSystem as struqture_two::mixed_systems::OperateOnMixedSystems>::current_number_bosonic_modes(&two_import).len();
            let bosonic_systems: Vec<Option<usize>> = vec![None; number_bosons];
            let number_fermions: usize = <struqture_two::mixed_systems::MixedLindbladOpenSystem as struqture_two::mixed_systems::OperateOnMixedSystems>::current_number_fermionic_modes(&two_import).len();
            let fermionic_systems: Vec<Option<usize>> = vec![None; number_fermions];
            let mut mixed_system: MixedLindbladOpenSystem = MixedLindbladOpenSystem::new(
                spin_systems.iter().cloned(),
                bosonic_systems.iter().cloned(),
                fermionic_systems.iter().cloned(),
            );
            let system = struqture_two::OpenSystem::system(&two_import);
            for (key, val) in struqture_two::OperateOnDensityMatrix::iter(system) {
                let value_string = key.to_string();
                let self_key = HermitianMixedProduct::from_str(&value_string).map_err(
                    |_err: StruqtureError| PyValueError::new_err(
                        "Trying to obtain struqture 1.x MixedLindbladOpenSystem from struqture 2.x MixedLindbladOpenSystem. Conversion failed. Was the right type passed to all functions?".to_string()
                ))?;

                let _ = mixed_system.system_mut().set(self_key, val.clone());
            }
            let noise = struqture_two::OpenSystem::noise(&two_import);
            for ((key_left, key_right), val) in struqture_two::OperateOnDensityMatrix::iter(noise) {
                let value_string_left = key_left.to_string();
                let value_string_right = key_right.to_string();
                let self_key = (MixedDecoherenceProduct::from_str(&value_string_left).map_err(
                    |_err: StruqtureError| PyValueError::new_err(
                        "Trying to obtain struqture 1.x MixedLindbladOpenSystem from struqture 2.x MixedLindbladOpenSystem. Conversion failed. Was the right type passed to all functions?".to_string()
                ))?, MixedDecoherenceProduct::from_str(&value_string_right).map_err(
                    |_err: StruqtureError| PyValueError::new_err(
                        "Trying to obtain struqture 1.x MixedLindbladOpenSystem from struqture 2.x MixedLindbladOpenSystem. Conversion failed. Was the right type passed to all functions?".to_string()
                ))?);

                let _ = mixed_system.noise_mut().set(self_key, val.clone());
            }

            Ok(MixedLindbladOpenSystemWrapper {
                internal: mixed_system,
            })
        })
    }
}
