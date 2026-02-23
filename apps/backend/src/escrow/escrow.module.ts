import { Module } from '@nestjs/common';
import { EscrowController } from 'src/modules/escrow/controllers/escrow.controller';
import { EscrowService } from 'src/modules/escrow/services/escrow.service';

@Module({
  controllers: [EscrowController],
  providers: [EscrowService],
})
export class EscrowModule {}
