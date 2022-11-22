use std::arch::asm;
use std::arch::global_asm;

global_asm!(
  ".globl test",
  "test:",
    "push rbp",
    "mov rbp, rsp",
    "mov rax, [rsp + 16]",
    "pop rbp",
    "ret"
);

extern {
  pub fn test();
}

pub fn call_test(i: i64) {
 let mut x = 0;
  unsafe {
    asm!(
      "push rbx",
      "mov rbx, rsp",
      "push {i}",
      "call test",
      "mov {x}, rax",
      "mov rsp, rbx",
      "pop rbx",
      i = in(reg) i,
      x = lateout(reg) x
    );
  }

  println!("X is: {}", x);
}

fn slice_swapper() {
  let mut ptr: *mut u8;
  let buffer = b"Test!".as_ptr() as *mut u8;
  
  unsafe {
    asm!(
      "mov rax, {0}",
      "mov rdi, rax",
      in(reg) buffer,
      lateout("rdi") ptr);
  }
  
  println!("eax: {:?}", std::str::from_utf8(unsafe { std::slice::from_raw_parts(ptr, 5) }).unwrap());
}

macro_rules! buffer {
  ($n:expr) => {
   &mut (vec![0_u8; $n])[..]
  };
}

fn deref_fill() {
  // stored as ascii in ebx, edx, ecx in order
  let name_buf = buffer!(16);
  
  // ebx is reserved, push to preserve value
  unsafe {
    asm!(
      "push rbx",
      "cpuid",
      "mov [rdi], ebx",
      "mov [rdi + 4], edx",
      "mov [rdi + 8], ecx",
      "mov [rdi + 12], {t}",
      "pop rbx",
      // little endian order
      t = in(reg) 0x5354554E,
      in("rdi") name_buf.as_mut_ptr(),
      // select cpuid 0; eax is clobbered
      inout("eax") 0 => _,
      // cpuid clobbers these registers too
      out("ecx") _,
      out("edx") _,
    );
  }

  let name = std::str::from_utf8(&name_buf).unwrap();
  println!("CPU Manufacturer ID: {}", name);
}

fn write_string(str: String) -> i32 {
  let mut res = 0;
  unsafe {
    asm!(
      "syscall",
      in("rdi") 1,
      in("rsi") str.as_ptr(),
      in("rdx") str.len(),
      inlateout("rax") 1 => res,
      // clobbered by syscall
      out("rcx") _,
      out("r11") _,
    );
  }
  res
}

fn exit(exit_code: i16) {
  unsafe {
    asm!(
      "mov rax, 60",
      "mov rdi, rsi",
      "syscall",
      in("rsi") exit_code,
      // clobbered by syscall
      out("rcx") _,
      out("r11") _,
    );
  }
}

fn getpid() -> i32 {
  let mut res;
  unsafe {
    asm!(
      "mov rax, 37",
      "syscall",
      "mov r8, rax",
      out("r8") res,
      // clobbered by syscall
      out("rcx") _,
      out("r11") _,
    );
  }

  res
}

fn main() {
  call_test(56);
  deref_fill();
  println!("PID is: {}", getpid());
  let bytes_written = write_string("Hello!\n".to_owned());
  println!("Bytes Written: {}", bytes_written);
  exit(1);
}
