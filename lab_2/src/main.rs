fn main() {
    println!("Hello, world!");

    let num1 = NumberWithUnit::unitless(2.64);
    let num1_again = NumberWithUnit::unitless(2.64);
    println!("This is num1: {:?}", num1);

    let num2 = NumberWithUnit::with_unit(3.15, String::from("m"));
    println!("This is num2: {:?}", num2);

    let num3 = NumberWithUnit::with_unit_from(num2, 9.99);
    println!("This is num3: {:?}", num3);

    let num4 = num3.mul(num1_again);
    println!("This is num4: {:?}", num4);

    let num5 = NumberWithUnit::with_unit(2.0, String::from("s"));
    let num6 = num1.div(num5);
    println!("This is num6: {:?}", num6);

    let num7 = NumberWithUnit::with_unit(3.15, String::from("m"));
    let num8 = NumberWithUnit::with_unit(3.15, String::from("m"));
    let mut dist = num7.add(num8);
    let more_dist = NumberWithUnit::with_unit(2.88888, String::from("m"));
    dist.add_in_place(&more_dist);
    println!("This is dist: {:?}", dist);

    let time = NumberWithUnit::with_unit(3.15, String::from("s"));
    dist.div_in_place(&time);
    println!("Dist is now velocity: {:?}", dist);

    dist.mul_in_place(&time);
    println!("Dist is back to being dist: {:?}", dist);

    let values_to_mult = [dist, more_dist, time];
    println!("{:?}", mul_vals(&values_to_mult[0..3]));
    println!("{:?}", mul_vals(&values_to_mult[0..3]));

    let values_in_vec: Vec<NumberWithUnit> = Vec::from(values_to_mult);
    let another_vec = values_in_vec.clone();
    println!("{:?}", mul_vals_vec(values_in_vec));
    println!("{:?}", mul_vals_vec(another_vec));
    // println!("{:?}", mul_vals_vec(values_in_vec));

    let str_slice: &str = "Abc";
    let string: String = String::from("Abc");

    let double1 = DoubleString::from_strs(&string, str_slice);
    double1.show();
    let double2 = DoubleString::from_strings(&string, &str_slice.to_string());
    double2.show();

}

#[derive(Clone, Debug, Default)]
struct NumberWithUnit {
    unit: String,
    value: f64
}

impl NumberWithUnit {
    fn unitless(value: f64) -> Self {
        Self { unit: String::new(), value }
    }
    fn with_unit(value: f64, unit: String) -> Self {
        Self { unit, value }
    }
    fn with_unit_from(other: Self, value: f64) -> Self {
        Self { unit: other.unit, value}
    }

    fn add(self, other: Self) -> Self {
        if self.unit != other.unit {
            panic!("The units in addition were different");
        }
        Self {
            value: self.value + other.value,
            unit: self.unit
        }
    }

    fn mul(self, other: Self) -> Self {
        let unit: String;
        if self.unit == other.unit {
            unit = format!("{}^2", self.unit);
        } else if self.unit.is_empty() || other.unit.is_empty() {
            unit = format!("{}{}", self.unit, other.unit);
        } else {
            unit = format!("{}*{}", self.unit, other.unit);
        }
        Self {
            value: self.value * other.value,
            unit
        }
    }

    fn div(self, other: Self) -> Self {
        let unit: String;
        if self.unit == other.unit {
            unit = String::from("");
        } else if self.unit.is_empty() {
            unit = format!("1/{}", other.unit);
        } else if other.unit.is_empty() {
            unit = self.unit
        } else {
            unit = format!("{}/{}", self.unit, other.unit);
        }
        Self {
            value: self.value / other.value,
            unit
        }
    }

    fn add_in_place(&mut self, other: &Self) {
        if self.unit != other.unit {
            panic!("The units in addition were different");
        }
        self.value += other.value;
    }

    fn mul_in_place(&mut self, other: &Self) {
        let unit: String;
        if self.unit == other.unit {
            unit = format!("{}^2", self.unit);
        } else if self.unit.is_empty() || other.unit.is_empty() {
            unit = format!("{}{}", self.unit, other.unit);
        } else {
            unit = format!("{}*{}", self.unit, other.unit);
        }
        self.value *= other.value;
        self.unit = unit;
    }

    fn div_in_place(&mut self, other: &Self) {
        if self.unit == other.unit {
            self.unit = String::from("");
        } else if self.unit.is_empty() {
            self.unit = format!("1/{}", other.unit);
        } else if other.unit.is_empty() {

        } else {
            self.unit = format!("{}/{}", self.unit, other.unit);
        }
        self.value /= other.value;
    }
}

fn mul_vals(vals: &[NumberWithUnit]) -> NumberWithUnit {
    let mut ret_val = NumberWithUnit::unitless(1.0);
    for num in vals {
        ret_val.mul_in_place(num);
    }
    ret_val
}

fn mul_vals_vec(vals: Vec<NumberWithUnit>) -> NumberWithUnit {
    let mut ret_val = NumberWithUnit::unitless(1.0);
    for num in vals {
        ret_val.mul_in_place(&num);
    }
    ret_val
}

struct DoubleString(String, String);

impl DoubleString {
    fn from_strs(str_1: &str, str_2: &str) -> Self {
        DoubleString(String::from(str_1), String::from(str_2))
    }

    fn from_strings(str_1: &String, str_2: &String) -> Self {
        DoubleString(str_1.to_string(), str_2.to_string())
    }

    fn show(&self) {
        println!("({}, {})", self.0, self.1);
    }
}