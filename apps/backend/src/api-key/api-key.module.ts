import { Module } from '@nestjs/common';
import { ApiKeysService } from './api-key.service';

@Module({
  providers: [ApiKeysService],
})
export class ApiKeyModule {}
