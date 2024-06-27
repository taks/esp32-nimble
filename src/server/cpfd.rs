use esp_idf_svc::sys as esp_idf_sys;
use num_enum::IntoPrimitive;

#[derive(Copy, Clone, PartialEq, Eq, Debug, IntoPrimitive)]
#[repr(u8)]
pub enum ChrFormat {
  Boolean = esp_idf_sys::BLE_GATT_CHR_FORMAT_BOOLEAN as _,
  Uint2 = esp_idf_sys::BLE_GATT_CHR_FORMAT_UINT2 as _,
  Uint4 = esp_idf_sys::BLE_GATT_CHR_FORMAT_UINT4 as _,
  Uint8 = esp_idf_sys::BLE_GATT_CHR_FORMAT_UINT8 as _,
  Uint12 = esp_idf_sys::BLE_GATT_CHR_FORMAT_UINT12 as _,
  Uint16 = esp_idf_sys::BLE_GATT_CHR_FORMAT_UINT16 as _,
  Uint24 = esp_idf_sys::BLE_GATT_CHR_FORMAT_UINT24 as _,
  Uint32 = esp_idf_sys::BLE_GATT_CHR_FORMAT_UINT32 as _,
  Uint48 = esp_idf_sys::BLE_GATT_CHR_FORMAT_UINT48 as _,
  Uint64 = esp_idf_sys::BLE_GATT_CHR_FORMAT_UINT64 as _,
  Uint128 = esp_idf_sys::BLE_GATT_CHR_FORMAT_UINT128 as _,
  Sint8 = esp_idf_sys::BLE_GATT_CHR_FORMAT_SINT8 as _,
  Sint12 = esp_idf_sys::BLE_GATT_CHR_FORMAT_SINT12 as _,
  Sint16 = esp_idf_sys::BLE_GATT_CHR_FORMAT_SINT16 as _,
  Sint24 = esp_idf_sys::BLE_GATT_CHR_FORMAT_SINT24 as _,
  Sint32 = esp_idf_sys::BLE_GATT_CHR_FORMAT_SINT32 as _,
  Sint48 = esp_idf_sys::BLE_GATT_CHR_FORMAT_SINT48 as _,
  Sint64 = esp_idf_sys::BLE_GATT_CHR_FORMAT_SINT64 as _,
  Sint128 = esp_idf_sys::BLE_GATT_CHR_FORMAT_SINT128 as _,
  Float32 = esp_idf_sys::BLE_GATT_CHR_FORMAT_FLOAT32 as _,
  Float64 = esp_idf_sys::BLE_GATT_CHR_FORMAT_FLOAT64 as _,
  Medfloat16 = esp_idf_sys::BLE_GATT_CHR_FORMAT_MEDFLOAT16 as _,
  Medfloat32 = esp_idf_sys::BLE_GATT_CHR_FORMAT_MEDFLOAT32 as _,
  Uint162 = esp_idf_sys::BLE_GATT_CHR_FORMAT_UINT16_2 as _,
  Utf8s = esp_idf_sys::BLE_GATT_CHR_FORMAT_UTF8S as _,
  Utf16s = esp_idf_sys::BLE_GATT_CHR_FORMAT_UTF16S as _,
  Struct = esp_idf_sys::BLE_GATT_CHR_FORMAT_STRUCT as _,
  Medasn1 = esp_idf_sys::BLE_GATT_CHR_FORMAT_MEDASN1 as _,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, IntoPrimitive)]
#[repr(u16)]
pub enum ChrUnit {
  /// Unitless
  Unitless = esp_idf_sys::BLE_GATT_CHR_UNIT_UNITLESS as _,
  // Length
  Metre = esp_idf_sys::BLE_GATT_CHR_UNIT_METRE as _,
  // Mass
  Kilogram = esp_idf_sys::BLE_GATT_CHR_UNIT_KILOGRAM as _,
  // Time
  Second = esp_idf_sys::BLE_GATT_CHR_UNIT_SECOND as _,
  // Electric Current
  Ampere = esp_idf_sys::BLE_GATT_CHR_UNIT_AMPERE as _,
  // Thermodynamic Temperature
  Kelvin = esp_idf_sys::BLE_GATT_CHR_UNIT_KELVIN as _,
  // Amount Of Substance
  Mole = esp_idf_sys::BLE_GATT_CHR_UNIT_MOLE as _,
  // Luminous Intensity
  Candela = esp_idf_sys::BLE_GATT_CHR_UNIT_CANDELA as _,
  // Area
  SquareMetres = esp_idf_sys::BLE_GATT_CHR_UNIT_SQUARE_METRES as _,
  // Volume
  CubicMetres = esp_idf_sys::BLE_GATT_CHR_UNIT_CUBIC_METRES as _,
  // Velocity
  MetresPerSecond = esp_idf_sys::BLE_GATT_CHR_UNIT_METRES_PER_SECOND as _,
  // Acceleration
  MetresPerSecondSquared = esp_idf_sys::BLE_GATT_CHR_UNIT_METRES_PER_SECOND_SQUARED as _,
  // Wavenumber
  ReciprocalMetre = esp_idf_sys::BLE_GATT_CHR_UNIT_RECIPROCAL_METRE as _,
  // Density
  KilogramPerCubicMetreDensity =
    esp_idf_sys::BLE_GATT_CHR_UNIT_KILOGRAM_PER_CUBIC_METRE_DENSITY as _,
  // Surface Density
  KilogramPerSquareMetre = esp_idf_sys::BLE_GATT_CHR_UNIT_KILOGRAM_PER_SQUARE_METRE as _,
  // Specific Volume
  CubicMetrePerKilogram = esp_idf_sys::BLE_GATT_CHR_UNIT_CUBIC_METRE_PER_KILOGRAM as _,
  // Current Density
  AmperePerSquareMetre = esp_idf_sys::BLE_GATT_CHR_UNIT_AMPERE_PER_SQUARE_METRE as _,
  // Magnetic Field Strength
  AmperePerMetre = esp_idf_sys::BLE_GATT_CHR_UNIT_AMPERE_PER_METRE as _,
  // Amount Concentration
  MolePerCubicMetre = esp_idf_sys::BLE_GATT_CHR_UNIT_MOLE_PER_CUBIC_METRE as _,
  // Mass Concentration
  KilogramPerCubicMetreMassConc =
    esp_idf_sys::BLE_GATT_CHR_UNIT_KILOGRAM_PER_CUBIC_METRE_MASS_CONC as _,
  // Luminance
  CandelaPerSquareMetre = esp_idf_sys::BLE_GATT_CHR_UNIT_CANDELA_PER_SQUARE_METRE as _,
  // Refractive Index
  RefractiveIndex = esp_idf_sys::BLE_GATT_CHR_UNIT_REFRACTIVE_INDEX as _,
  // Relative Permeability
  RelativePermeability = esp_idf_sys::BLE_GATT_CHR_UNIT_RELATIVE_PERMEABILITY as _,
  // Plane Angle
  Radian = esp_idf_sys::BLE_GATT_CHR_UNIT_RADIAN as _,
  // Solid Angle
  Steradian = esp_idf_sys::BLE_GATT_CHR_UNIT_STERADIAN as _,
  // Frequency
  Hertz = esp_idf_sys::BLE_GATT_CHR_UNIT_HERTZ as _,
  // Force
  Newton = esp_idf_sys::BLE_GATT_CHR_UNIT_NEWTON as _,
  // Pressure
  Pascal = esp_idf_sys::BLE_GATT_CHR_UNIT_PASCAL as _,
  // Energy
  Joule = esp_idf_sys::BLE_GATT_CHR_UNIT_JOULE as _,
  // Power
  Watt = esp_idf_sys::BLE_GATT_CHR_UNIT_WATT as _,
  // Electric Charge
  Coulomb = esp_idf_sys::BLE_GATT_CHR_UNIT_COULOMB as _,
  // Electric Potential Difference
  Volt = esp_idf_sys::BLE_GATT_CHR_UNIT_VOLT as _,
  // Capacitance
  Farad = esp_idf_sys::BLE_GATT_CHR_UNIT_FARAD as _,
  // Electric Resistance
  Ohm = esp_idf_sys::BLE_GATT_CHR_UNIT_OHM as _,
  // Electric Conductance
  Siemens = esp_idf_sys::BLE_GATT_CHR_UNIT_SIEMENS as _,
  // Magnetic Flux
  Weber = esp_idf_sys::BLE_GATT_CHR_UNIT_WEBER as _,
  // Magnetic Flux Density
  Tesla = esp_idf_sys::BLE_GATT_CHR_UNIT_TESLA as _,
  // Inductance
  Henry = esp_idf_sys::BLE_GATT_CHR_UNIT_HENRY as _,
  // celsius Temperature
  DegreeCelsius = esp_idf_sys::BLE_GATT_CHR_UNIT_DEGREE_CELSIUS as _,
  // Luminous Flux
  Lumen = esp_idf_sys::BLE_GATT_CHR_UNIT_LUMEN as _,
  // Illuminance
  Lux = esp_idf_sys::BLE_GATT_CHR_UNIT_LUX as _,
  // Activity Referred To A Radionuclide
  Becquerel = esp_idf_sys::BLE_GATT_CHR_UNIT_BECQUEREL as _,
  // Absorbed Dose
  Gray = esp_idf_sys::BLE_GATT_CHR_UNIT_GRAY as _,
  // Dose Equivalent
  Sievert = esp_idf_sys::BLE_GATT_CHR_UNIT_SIEVERT as _,
  // Catalytic Activity
  Katal = esp_idf_sys::BLE_GATT_CHR_UNIT_KATAL as _,
  // Dynamic Viscosity
  PascalSecond = esp_idf_sys::BLE_GATT_CHR_UNIT_PASCAL_SECOND as _,
  // Moment Of Force
  NewtonMetre = esp_idf_sys::BLE_GATT_CHR_UNIT_NEWTON_METRE as _,
  // Surface Tension
  NewtonPerMetre = esp_idf_sys::BLE_GATT_CHR_UNIT_NEWTON_PER_METRE as _,
  // Angular Velocity
  RadianPerSecond = esp_idf_sys::BLE_GATT_CHR_UNIT_RADIAN_PER_SECOND as _,
  // Angular Acceleration
  RadianPerSecondSquared = esp_idf_sys::BLE_GATT_CHR_UNIT_RADIAN_PER_SECOND_SQUARED as _,
  // Heat Flux Density
  WattPerSquareMetreHeat = esp_idf_sys::BLE_GATT_CHR_UNIT_WATT_PER_SQUARE_METRE_HEAT as _,
  // Heat Capacity
  JoulePerKelvin = esp_idf_sys::BLE_GATT_CHR_UNIT_JOULE_PER_KELVIN as _,
  // Specific Heat Capacity
  JoulePerKilogramKelvin = esp_idf_sys::BLE_GATT_CHR_UNIT_JOULE_PER_KILOGRAM_KELVIN as _,
  // Specific Energy
  JoulePerKilogram = esp_idf_sys::BLE_GATT_CHR_UNIT_JOULE_PER_KILOGRAM as _,
  // Thermal Conductivity
  WattPerMetreKelvin = esp_idf_sys::BLE_GATT_CHR_UNIT_WATT_PER_METRE_KELVIN as _,
  // Energy Density
  JoulePerCubicMetre = esp_idf_sys::BLE_GATT_CHR_UNIT_JOULE_PER_CUBIC_METRE as _,
  // Electric Field Strength
  VoltPerMetre = esp_idf_sys::BLE_GATT_CHR_UNIT_VOLT_PER_METRE as _,
  // Electric Charge Density
  CoulombPerCubicMetre = esp_idf_sys::BLE_GATT_CHR_UNIT_COULOMB_PER_CUBIC_METRE as _,
  // Surface Charge Density
  CoulombPerSquareMetreCharge = esp_idf_sys::BLE_GATT_CHR_UNIT_COULOMB_PER_SQUARE_METRE_CHARGE as _,
  // Electric Flux Density
  CoulombPerSquareMetreFlux = esp_idf_sys::BLE_GATT_CHR_UNIT_COULOMB_PER_SQUARE_METRE_FLUX as _,
  // Permittivity
  FaradPerMetre = esp_idf_sys::BLE_GATT_CHR_UNIT_FARAD_PER_METRE as _,
  // Permeability
  HenryPerMetre = esp_idf_sys::BLE_GATT_CHR_UNIT_HENRY_PER_METRE as _,
  // Molar Energy
  JoulePerMole = esp_idf_sys::BLE_GATT_CHR_UNIT_JOULE_PER_MOLE as _,
  // Molar Entropy
  JoulePerMoleKelvin = esp_idf_sys::BLE_GATT_CHR_UNIT_JOULE_PER_MOLE_KELVIN as _,
  // Exposure
  CoulombPerKilogram = esp_idf_sys::BLE_GATT_CHR_UNIT_COULOMB_PER_KILOGRAM as _,
  // Absorbed Dose Rate
  GrayPerSecond = esp_idf_sys::BLE_GATT_CHR_UNIT_GRAY_PER_SECOND as _,
  // Radiant Intensity
  WattPerSteradian = esp_idf_sys::BLE_GATT_CHR_UNIT_WATT_PER_STERADIAN as _,
  // Radiance
  WattPerSquareMetreSteradian = esp_idf_sys::BLE_GATT_CHR_UNIT_WATT_PER_SQUARE_METRE_STERADIAN as _,
  // Catalytic Activity Concentration
  KatalPerCubicMetre = esp_idf_sys::BLE_GATT_CHR_UNIT_KATAL_PER_CUBIC_METRE as _,
  // Time
  Minute = esp_idf_sys::BLE_GATT_CHR_UNIT_MINUTE as _,
  // Time
  Hour = esp_idf_sys::BLE_GATT_CHR_UNIT_HOUR as _,
  // Time
  Day = esp_idf_sys::BLE_GATT_CHR_UNIT_DAY as _,
  // Plane Angle
  Degree = esp_idf_sys::BLE_GATT_CHR_UNIT_DEGREE as _,
  // Plane Angle
  MinuteAngle = esp_idf_sys::BLE_GATT_CHR_UNIT_MINUTE_ANGLE as _,
  // Plane Angle
  SecondAngle = esp_idf_sys::BLE_GATT_CHR_UNIT_SECOND_ANGLE as _,
  // Area
  Hectare = esp_idf_sys::BLE_GATT_CHR_UNIT_HECTARE as _,
  // Volume
  Litre = esp_idf_sys::BLE_GATT_CHR_UNIT_LITRE as _,
  // Mass
  Tonne = esp_idf_sys::BLE_GATT_CHR_UNIT_TONNE as _,
  // Pressure
  Bar = esp_idf_sys::BLE_GATT_CHR_UNIT_BAR as _,
  // Pressure
  MillimetreOfMercury = esp_idf_sys::BLE_GATT_CHR_UNIT_MILLIMETRE_OF_MERCURY as _,
  // Length
  Angstrom = esp_idf_sys::BLE_GATT_CHR_UNIT_ANGSTROM as _,
  // Length
  NauticalMile = esp_idf_sys::BLE_GATT_CHR_UNIT_NAUTICAL_MILE as _,
  // Area
  Barn = esp_idf_sys::BLE_GATT_CHR_UNIT_BARN as _,
  // Velocity
  Knot = esp_idf_sys::BLE_GATT_CHR_UNIT_KNOT as _,
  // Logarithmic Radio Quantity
  Neper = esp_idf_sys::BLE_GATT_CHR_UNIT_NEPER as _,
  // Logarithmic Radio Quantity
  Bel = esp_idf_sys::BLE_GATT_CHR_UNIT_BEL as _,
  // Length
  Yard = esp_idf_sys::BLE_GATT_CHR_UNIT_YARD as _,
  // Length
  Parsec = esp_idf_sys::BLE_GATT_CHR_UNIT_PARSEC as _,
  // Length
  Inch = esp_idf_sys::BLE_GATT_CHR_UNIT_INCH as _,
  // Length
  Foot = esp_idf_sys::BLE_GATT_CHR_UNIT_FOOT as _,
  // Length
  Mile = esp_idf_sys::BLE_GATT_CHR_UNIT_MILE as _,
  // Pressure
  PoundForcePerSquareInch = esp_idf_sys::BLE_GATT_CHR_UNIT_POUND_FORCE_PER_SQUARE_INCH as _,
  // Velocity
  KilometrePerHour = esp_idf_sys::BLE_GATT_CHR_UNIT_KILOMETRE_PER_HOUR as _,
  // Velocity
  MilePerHour = esp_idf_sys::BLE_GATT_CHR_UNIT_MILE_PER_HOUR as _,
  // Angular Velocity
  RevolutionPerMinute = esp_idf_sys::BLE_GATT_CHR_UNIT_REVOLUTION_PER_MINUTE as _,
  // Energy
  GramCalorie = esp_idf_sys::BLE_GATT_CHR_UNIT_GRAM_CALORIE as _,
  // Energy
  KilogramCalorie = esp_idf_sys::BLE_GATT_CHR_UNIT_KILOGRAM_CALORIE as _,
  // Energy
  KilowattHour = esp_idf_sys::BLE_GATT_CHR_UNIT_KILOWATT_HOUR as _,
  // Thermodynamic Temperature
  DegreeFahrenheit = esp_idf_sys::BLE_GATT_CHR_UNIT_DEGREE_FAHRENHEIT as _,
  // Percentage
  Percentage = esp_idf_sys::BLE_GATT_CHR_UNIT_PERCENTAGE as _,
  // Per Mille
  PerMille = esp_idf_sys::BLE_GATT_CHR_UNIT_PER_MILLE as _,
  // Period
  BeatsPerMinute = esp_idf_sys::BLE_GATT_CHR_UNIT_BEATS_PER_MINUTE as _,
  // Electric Charge
  AmpereHours = esp_idf_sys::BLE_GATT_CHR_UNIT_AMPERE_HOURS as _,
  // Mass Density
  MilligramPerDecilitre = esp_idf_sys::BLE_GATT_CHR_UNIT_MILLIGRAM_PER_DECILITRE as _,
  // Mass Density
  MillimolePerLitre = esp_idf_sys::BLE_GATT_CHR_UNIT_MILLIMOLE_PER_LITRE as _,
  // Time
  Year = esp_idf_sys::BLE_GATT_CHR_UNIT_YEAR as _,
  // Time
  Month = esp_idf_sys::BLE_GATT_CHR_UNIT_MONTH as _,
  // Concentration
  CountPerCubicMetre = esp_idf_sys::BLE_GATT_CHR_UNIT_COUNT_PER_CUBIC_METRE as _,
  // Irradiance
  WattPerSquareMetreIrradiance =
    esp_idf_sys::BLE_GATT_CHR_UNIT_WATT_PER_SQUARE_METRE_IRRADIANCE as _,
  // Milliliter
  PerKilogramPerMinute = esp_idf_sys::BLE_GATT_CHR_UNIT_PER_KILOGRAM_PER_MINUTE as _,
  // Mass
  Pound = esp_idf_sys::BLE_GATT_CHR_UNIT_POUND as _,
  // Metabolic Equivalent
  MetabolicEquivalent = esp_idf_sys::BLE_GATT_CHR_UNIT_METABOLIC_EQUIVALENT as _,
  // Step
  PerMinuteStep = esp_idf_sys::BLE_GATT_CHR_UNIT_PER_MINUTE_STEP as _,
  // Stroke
  PerMinuteStroke = esp_idf_sys::BLE_GATT_CHR_UNIT_PER_MINUTE_STROKE as _,
  // Pace
  KilometrePerMinute = esp_idf_sys::BLE_GATT_CHR_UNIT_KILOMETRE_PER_MINUTE as _,
  // Luminous Efficacy
  LumenPerWatt = esp_idf_sys::BLE_GATT_CHR_UNIT_LUMEN_PER_WATT as _,
  // Luminous Energy
  LumenHour = esp_idf_sys::BLE_GATT_CHR_UNIT_LUMEN_HOUR as _,
  // Luminous Exposure
  LuxHour = esp_idf_sys::BLE_GATT_CHR_UNIT_LUX_HOUR as _,
  // Mass Flow
  GramPerSecond = esp_idf_sys::BLE_GATT_CHR_UNIT_GRAM_PER_SECOND as _,
  // Volume Flow
  LitrePerSecond = esp_idf_sys::BLE_GATT_CHR_UNIT_LITRE_PER_SECOND as _,
  // Sound Pressure
  Decibel = esp_idf_sys::BLE_GATT_CHR_UNIT_DECIBEL as _,
  // Concentration
  PartsPerMillion = esp_idf_sys::BLE_GATT_CHR_UNIT_PARTS_PER_MILLION as _,
  // Concentration
  PartsPerBillion = esp_idf_sys::BLE_GATT_CHR_UNIT_PARTS_PER_BILLION as _,
  // Mass Density Rate
  MilligramPerDecilitrePerMinute =
    esp_idf_sys::BLE_GATT_CHR_UNIT_MILLIGRAM_PER_DECILITRE_PER_MINUTE as _,
  // Electrical Apparent Energy
  KilovoltAmpereHour = esp_idf_sys::BLE_GATT_CHR_UNIT_KILOVOLT_AMPERE_HOUR as _,
  // Electrical Apparent Power
  VoltAmpere = esp_idf_sys::BLE_GATT_CHR_UNIT_VOLT_AMPERE as _,
}

pub struct Cpfd {
  /// Format of the value of this characteristic.
  pub format: ChrFormat,
  /// Exponent field. Multiplies the value to 10^exponent.
  pub exponent: i8,
  /// The unit of this characteristic.
  pub unit: ChrUnit,
  /// The name space of the description.
  pub name_space: u8,
  /// The description of this characteristic. Depends on name space.
  pub description: u16,
}
