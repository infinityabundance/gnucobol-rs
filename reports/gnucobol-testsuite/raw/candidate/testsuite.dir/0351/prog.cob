
       identification division.
       program-id. prog.
       environment division.
       data division.
       working-storage section.
       01  item-01.
           05  item-05-a     pointer.
           05  item-05-b     pic x(01).
       01 myk-01 constant           as      length of item-01.
       01 myk-02 constant is global as      length item-05-a.
       01 myk-03 constant    global as      length of pointer. *> extension
       01 myk-04 constant    global as byte-length of item-01.
       01 myk-05 constant is global as byte-length item-05-a.
       01 myk-06 constant           as byte-length of pointer. *> extension
