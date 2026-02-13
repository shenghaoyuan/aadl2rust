// #include <stdio.h>
// #include <request.h>
// #include <deployment.h>
// #include <po_hi_storage.h>
// #include <po_hi_gqueue.h>

// #define FILENAME "pinger.dat"

// __po_hi_storage_file_t myfile_read;
// __po_hi_storage_file_t myfile_write;

// void user_produce_pkts_init ()
// {
//   printf ("*** INIT DATA PRODUCER ***\n");
//   fflush (stdout);

//   if (__po_hi_storage_file_open (FILENAME, &myfile_write) != __PO_HI_SUCCESS)
//     {
//       printf ("*** /!\\ ERROR WHEN OPENING THE FILE %s /!\\ ***\n", FILENAME);
//       fflush (stdout);
//     }

//   if (__po_hi_storage_file_create (&myfile_write) != __PO_HI_SUCCESS)
//     {
//       printf ("*** /!\\ ERROR WHEN CREATING THE FILE %s /!\\ ***\n", FILENAME);
//       fflush (stdout);
//     }

//   if (__po_hi_storage_file_open (FILENAME, &myfile_read) != __PO_HI_SUCCESS)
//     {
//       printf ("*** /!\\ ERROR WHEN OPENING THE FILE %s /!\\ ***\n", FILENAME);
//       fflush (stdout);
//     }
// }

// void user_produce_pkts ()
// {
//   static int p = 0;
//   int ret;

//   __po_hi_request_t pkt;
//   pkt.vars.pinger_global_data_source.pinger_global_data_source = p;
//   pkt.port = pinger_global_data_source;

//   printf ("*** PRODUCE PKT WITH VALUE *** %d\n", p);
//   fflush (stdout);
//   p++;

//   ret = __po_hi_storage_file_write (&myfile_write, &pkt, sizeof (__po_hi_request_t));

//   if (ret != __PO_HI_SUCCESS)
//     {
//       printf ("*** /!\\ ERROR WHEN WRITING A PACKET IN THE FILE (ret=%d) /!\\ ***\n", ret);
//       fflush (stdout);
//     }
//  }

// void user_do_ping_spg ()
// {

//   int ret;
//   __po_hi_request_t pkt;

//   ret = __po_hi_storage_file_read (&myfile_read, &pkt, sizeof (__po_hi_request_t));


//   if (ret != __PO_HI_SUCCESS) {
//     printf ("*** /!\\ ERROR WHEN READING A PACKET FROM FILE /!\\ ***\n");
//     fflush (stdout);

//     if (ret == __PO_HI_UNAVAILABLE) {
//       printf ("*** /!\\ ;_; NO PACKET AVAILABLE AT THIS TIME ;_; /!\\ ***\n");
//       fflush (stdout);
//     }
//   } else {
//     printf ("*** SENDING PKT *** \n");
//     fflush (stdout);
//     __po_hi_gqueue_store_out (node_a_pinger_k, pinger_local_data_source, &(pkt));
//   }
//   fflush (stdout);
// }

// void recover (void)
// {
//   printf ("*** RECOVER ACTION ***\n");
//   fflush (stdout);
// }

// void user_ping_spg (int i)
// {
//   printf ("*** PING *** %d\n" ,i);
//   fflush (stdout);
// }

#include <stdio.h>
#include <stdint.h>
#include <stdlib.h>
#include "pinger.h"

#define FILENAME "pinger.dat"

// 全局文件指针
FILE *file_ptr = NULL;

void user_produce_pkts_init ()
{
  printf ("*** INIT DATA PRODUCER ***\n");
  
  // 以 "ab+" 模式打开：追加写入，同时支持读取
  // 如果文件不存在则创建
  file_ptr = fopen(FILENAME, "ab+");
  
  if (file_ptr == NULL) {
      printf ("*** /!\\ ERROR WHEN OPENING THE FILE %s /!\\ ***\n", FILENAME);
      perror("fopen");
  } else {
      printf ("*** FILE OPENED SUCCESSFULLY ***\n");
  }
  fflush (stdout);
}

void user_produce_pkts ()
{
  static int32_t p = 0;
  
  if (file_ptr == NULL) {
      // 尝试重新打开
      file_ptr = fopen(FILENAME, "ab+");
      if (file_ptr == NULL) return;
  }

  printf ("*** PRODUCE PKT WITH VALUE *** %d\n", p);
  
  // 移动到文件末尾进行追加
  fseek(file_ptr, 0, SEEK_END);
  size_t written = fwrite(&p, sizeof(int32_t), 1, file_ptr);
  fflush(file_ptr); // 确保写入磁盘

  if (written != 1) {
      printf ("*** /!\\ ERROR WHEN WRITING TO FILE /!\\ ***\n");
  }

  p++;
  fflush (stdout);
}

// 修改函数签名：接受一个输出指针
void user_do_ping_spg (int32_t* data_source)
{
  static long read_offset = 0;
  int32_t val;
  
  if (file_ptr == NULL) {
      file_ptr = fopen(FILENAME, "rb"); // 尝试只读打开
      if (file_ptr == NULL) return;
  }

  // 移动到上次读取的位置
  fseek(file_ptr, read_offset, SEEK_SET);
  
  size_t read_count = fread(&val, sizeof(int32_t), 1, file_ptr);

  if (read_count == 1) {
      printf ("*** SENDING PKT (Read from file) *** : %d\n", val);
      *data_source = val; // 将数据传给 AADL 端口
      
      // 更新读取位置
      read_offset = ftell(file_ptr);
  } else {
      // 没有新数据，这里可以发送一个特定值或者不做任何事
      // 这里的逻辑稍微简单处理：如果没有读到，就发一个 -1 或者保持上一个值
      // 实际上 AADL 生成器如果不赋值给 *data_source，可能会发送随机值
      // 我们这里暂时不发送有效数据，但在 AADL event data port 中总会有数据流出
      // 可以在这里做个标记，例如：
      printf ("*** /!\\ NO NEW PACKET AVAILABLE ;_; /!\\ ***\n");
      // *data_source = -1; // 可选
  }
  fflush (stdout);
}

void recover (void)
{
  printf ("*** RECOVER ACTION ***\n");
  fflush (stdout);
}

void user_ping_spg (int32_t i)
{
  printf ("*** PING (Received) *** %d\n" ,i);
  fflush (stdout);
}